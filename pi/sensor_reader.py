#!/usr/bin/env python3
"""
MAX30102 heart rate sensor reader for Raspberry Pi.
Reads IR PPG data over I2C, computes BPM using peak detection,
and POSTs each reading to the stress monitor API.
"""

import time
import struct
import requests
import smbus2
from collections import deque

# ---------------------------------------------------------------------------
# Configuration — edit these before running
# ---------------------------------------------------------------------------
API_BASE_URL    = ""
API_KEY         = "somereallysecurecryptographickeyofsomesort"
READING_INTERVAL = 1        # seconds between API POSTs
I2C_BUS         = 1         # /dev/i2c-1 on most Pi models
# ---------------------------------------------------------------------------

# MAX30102 I2C address and register map
MAX30102_ADDR   = 0x57

REG_INTR_STATUS_1   = 0x00
REG_INTR_ENABLE_1   = 0x02
REG_INTR_ENABLE_2   = 0x03
REG_FIFO_WR_PTR     = 0x04
REG_OVF_COUNTER     = 0x05
REG_FIFO_RD_PTR     = 0x06
REG_FIFO_DATA       = 0x07
REG_FIFO_CONFIG     = 0x08
REG_MODE_CONFIG     = 0x09
REG_SPO2_CONFIG     = 0x0A
REG_LED1_PA         = 0x0C   # Red LED
REG_LED2_PA         = 0x0D   # IR LED
REG_PART_ID         = 0xFF

# Sensor operating parameters
SAMPLE_RATE     = 100   # Hz (matches REG_SPO2_CONFIG below)
WINDOW_SECONDS  = 5     # seconds of IR data used to compute BPM
MIN_PEAKS       = 2     # require at least this many peaks for a valid reading

ir_buffer: deque = deque(maxlen=SAMPLE_RATE * WINDOW_SECONDS)


# ---------------------------------------------------------------------------
# MAX30102 driver
# ---------------------------------------------------------------------------

def sensor_reset(bus: smbus2.SMBus) -> None:
    """Soft-reset the device and wait for it to come back up."""
    bus.write_byte_data(MAX30102_ADDR, REG_MODE_CONFIG, 0x40)
    time.sleep(0.01)
    # Wait until the RESET bit clears
    for _ in range(10):
        if not (bus.read_byte_data(MAX30102_ADDR, REG_MODE_CONFIG) & 0x40):
            return
        time.sleep(0.01)
    raise RuntimeError("MAX30102 did not come out of reset")


def sensor_init(bus: smbus2.SMBus) -> None:
    """Configure the MAX30102 for SpO2 (dual-LED) mode at 100 sps."""
    # FIFO: no sample averaging, roll-over enabled
    bus.write_byte_data(MAX30102_ADDR, REG_FIFO_CONFIG, 0x1F)
    # SpO2 mode (RED + IR), not shutdown
    bus.write_byte_data(MAX30102_ADDR, REG_MODE_CONFIG, 0x03)
    # SPO2_ADC_RGE=2 (4096nA), SPO2_SR=100sps, LED_PW=411µs (18-bit)
    bus.write_byte_data(MAX30102_ADDR, REG_SPO2_CONFIG, 0x27)
    # LED currents: ~3 mA each (low enough to avoid ADC saturation)
    bus.write_byte_data(MAX30102_ADDR, REG_LED1_PA, 0x0F)
    bus.write_byte_data(MAX30102_ADDR, REG_LED2_PA, 0x0F)
    # Clear FIFO pointers
    bus.write_byte_data(MAX30102_ADDR, REG_FIFO_WR_PTR, 0x00)
    bus.write_byte_data(MAX30102_ADDR, REG_OVF_COUNTER, 0x00)
    bus.write_byte_data(MAX30102_ADDR, REG_FIFO_RD_PTR, 0x00)


def verify_init(bus: smbus2.SMBus) -> None:
    """Read back key registers to confirm writes landed."""
    mode    = bus.read_byte_data(MAX30102_ADDR, REG_MODE_CONFIG)
    fifo_cf = bus.read_byte_data(MAX30102_ADDR, REG_FIFO_CONFIG)
    spo2_cf = bus.read_byte_data(MAX30102_ADDR, REG_SPO2_CONFIG)
    led1    = bus.read_byte_data(MAX30102_ADDR, REG_LED1_PA)
    led2    = bus.read_byte_data(MAX30102_ADDR, REG_LED2_PA)
    print(f"  MODE=0x{mode:02X} (want 0x03)  "
          f"FIFO_CFG=0x{fifo_cf:02X} (want 0x1F)  "
          f"SPO2_CFG=0x{spo2_cf:02X} (want 0x27)  "
          f"LED1=0x{led1:02X}  LED2=0x{led2:02X}")
    if mode & 0x07 != 0x03:
        print("[WARN]  MODE register did not set correctly — sampling will not start")


def read_fifo_samples(bus: smbus2.SMBus) -> list[int]:
    """
    Read all available samples from the FIFO.
    Returns a list of IR values (18-bit, right-justified).
    Each sample in SpO2 mode is 6 bytes: 3 bytes RED + 3 bytes IR.
    """
    wr_ptr = bus.read_byte_data(MAX30102_ADDR, REG_FIFO_WR_PTR) & 0x1F
    rd_ptr = bus.read_byte_data(MAX30102_ADDR, REG_FIFO_RD_PTR) & 0x1F
    num_samples = (wr_ptr - rd_ptr) & 0x1F

    ir_values = []
    for _ in range(num_samples):
        raw = bus.read_i2c_block_data(MAX30102_ADDR, REG_FIFO_DATA, 6)
        # Bytes 0-2 are RED, bytes 3-5 are IR; mask upper 2 bits (18-bit ADC)
        ir = ((raw[3] << 16) | (raw[4] << 8) | raw[5]) & 0x3FFFF
        ir_values.append(ir)

    return ir_values


# ---------------------------------------------------------------------------
# BPM computation
# ---------------------------------------------------------------------------

def moving_average(data: list[float], window: int) -> list[float]:
    result = []
    for i in range(len(data)):
        start = max(0, i - window + 1)
        result.append(sum(data[start:i + 1]) / (i - start + 1))
    return result


def compute_bpm(ir_data: list[int]) -> float | None:
    """
    Estimate BPM from a window of IR PPG samples.

    Steps:
      1. Remove DC baseline using a moving average (low-pass).
      2. Find peaks in the AC signal above a dynamic threshold.
      3. Convert average inter-peak spacing to BPM.

    Returns BPM as a float, or None if a reliable reading cannot be computed.
    """
    if len(ir_data) < SAMPLE_RATE * 2:
        return None

    data = [float(v) for v in ir_data]

    # Low-pass: moving average over ~0.5 s
    lp_window = SAMPLE_RATE // 2
    dc = moving_average(data, lp_window)

    # AC component
    ac = [data[i] - dc[i] for i in range(len(data))]

    # Dynamic threshold: 35 % of peak AC amplitude
    ac_max = max(ac)
    if ac_max <= 0:
        return None
    threshold = ac_max * 0.35

    # Minimum spacing between peaks: 60 s / 220 BPM ≈ 0.27 s
    min_dist = int(SAMPLE_RATE * 0.27)

    peaks: list[int] = []
    i = 1
    while i < len(ac) - 1:
        if ac[i] > ac[i - 1] and ac[i] > ac[i + 1] and ac[i] > threshold:
            if not peaks or (i - peaks[-1]) >= min_dist:
                peaks.append(i)
        i += 1

    if len(peaks) < MIN_PEAKS:
        return None

    intervals = [peaks[k + 1] - peaks[k] for k in range(len(peaks) - 1)]
    avg_interval = sum(intervals) / len(intervals)
    bpm = 60.0 * SAMPLE_RATE / avg_interval

    # Plausibility check
    if 40.0 <= bpm <= 220.0:
        return bpm
    return None


# ---------------------------------------------------------------------------
# API
# ---------------------------------------------------------------------------

def post_reading(bpm: float) -> None:
    payload = {
        "key": API_KEY,
        "hr": round(bpm, 1),
        "timestamp": int(time.time()),
    }
    try:
        resp = requests.post(f"{API_BASE_URL}/info", json=payload, timeout=5, verify=False)
        if resp.status_code == 200:
            print(f"[OK]    Posted {bpm:.1f} BPM")
        elif resp.status_code == 401:
            print("[ERROR] Unauthorized — check API_KEY")
        else:
            print(f"[WARN]  Server returned {resp.status_code}")
    except requests.RequestException as exc:
        print(f"[WARN]  HTTP error: {exc}")


# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------

def main() -> None:
    bus = smbus2.SMBus(I2C_BUS)

    part_id = bus.read_byte_data(MAX30102_ADDR, REG_PART_ID)
    if part_id != 0x15:
        print(f"[WARN]  Unexpected part ID: 0x{part_id:02X} (expected 0x15). Continuing anyway.")

    print("Resetting MAX30102…")
    sensor_reset(bus)
    sensor_init(bus)
    time.sleep(0.1)   # let the sensor stabilise before first FIFO read
    print("Register check after init:")
    verify_init(bus)
    print(f"MAX30102 initialised. Sampling at {SAMPLE_RATE} Hz. "
          f"Posting every {READING_INTERVAL} s.")
    print("Place your finger on the sensor — allow ~5 s for stabilisation.\n")

    last_post = time.monotonic()
    warm_up_until = time.monotonic() + 5.0   # discard first 5 s of data

    while True:
        try:
            samples = read_fifo_samples(bus)
        except OSError as exc:
            print(f"[ERROR] I2C read failed: {exc}. Retrying in 1 s…")
            time.sleep(1)
            continue

        now = time.monotonic()

        wr = bus.read_byte_data(MAX30102_ADDR, REG_FIFO_WR_PTR) & 0x1F
        rd = bus.read_byte_data(MAX30102_ADDR, REG_FIFO_RD_PTR) & 0x1F
        ir_latest = samples[-1] if samples else 0
        finger = "finger detected" if ir_latest > 50_000 else "NO FINGER"
        print(f"\r  IR: {ir_latest:>8}  |  WR={wr} RD={rd}  |  samples: {len(samples)}  |  {finger}    ",
              end="", flush=True)

        if now >= warm_up_until:
            ir_buffer.extend(samples)

        if (now - last_post) >= READING_INTERVAL and len(ir_buffer) >= SAMPLE_RATE * 2:
            bpm = compute_bpm(list(ir_buffer))
            if bpm is not None:
                print()  # newline before the POST line
                post_reading(bpm)
            else:
                print("\n[INFO]  Not enough signal yet — waiting for more data…")
            last_post = now

        # Pace the polling loop to avoid hammering the I2C bus
        time.sleep(1 / SAMPLE_RATE)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nStopped.")
