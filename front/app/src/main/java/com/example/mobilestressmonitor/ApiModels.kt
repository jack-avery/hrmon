package com.example.mobilestressmonitor

import com.google.gson.annotations.SerializedName

data class InfoResponse(
    @SerializedName("user_state") val userState: String,
    @SerializedName("avg_hr") val avgHr: Double,
    @SerializedName("hr_data") val hrData: List<HrReading>
)

data class HrReading(
    @SerializedName("timestamp") val timestamp: Long,
    @SerializedName("hr") val hr: Double
)

data class FlushRequest(
    @SerializedName("key") val key: String
)
