package com.example.mobilestressmonitor

import android.graphics.Color
import android.os.Bundle
import android.view.View
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ProgressBar
import android.widget.TextView
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.lifecycleScope
import com.google.android.material.card.MaterialCardView
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

class MainActivity : AppCompatActivity() {

    private val viewModel: DashboardViewModel by viewModels()

    private lateinit var loadingView: LinearLayout
    private lateinit var errorView: LinearLayout
    private lateinit var contentView: LinearLayout
    private lateinit var errorMessageText: TextView
    private lateinit var retryButton: Button
    private lateinit var baselineHrText: TextView
    private lateinit var currentHrText: TextView
    private lateinit var stressLevelText: TextView
    private lateinit var stressCard: MaterialCardView
    private lateinit var lastUpdatedText: TextView

    private val timeFormat = SimpleDateFormat("h:mm:ss a", Locale.getDefault())

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        loadingView = findViewById(R.id.loadingView)
        errorView = findViewById(R.id.errorView)
        contentView = findViewById(R.id.contentView)
        errorMessageText = findViewById(R.id.errorMessageText)
        retryButton = findViewById(R.id.retryButton)
        baselineHrText = findViewById(R.id.baselineHrText)
        currentHrText = findViewById(R.id.currentHrText)
        stressLevelText = findViewById(R.id.stressLevelText)
        stressCard = findViewById(R.id.stressCard)
        lastUpdatedText = findViewById(R.id.lastUpdatedText)

        retryButton.setOnClickListener { viewModel.retry() }

        lifecycleScope.launch {
            viewModel.state.collect { state ->
                when (state) {
                    is DashboardState.Loading -> showLoading()
                    is DashboardState.Success -> showContent(state.data)
                    is DashboardState.Error -> showError(state.message)
                }
            }
        }
    }

    private fun showLoading() {
        loadingView.visibility = View.VISIBLE
        errorView.visibility = View.GONE
        contentView.visibility = View.GONE
    }

    private fun showError(message: String) {
        loadingView.visibility = View.GONE
        errorView.visibility = View.VISIBLE
        contentView.visibility = View.GONE
        errorMessageText.text = "Error: $message"
    }

    private fun showContent(data: InfoResponse) {
        loadingView.visibility = View.GONE
        errorView.visibility = View.GONE
        contentView.visibility = View.VISIBLE

        baselineHrText.text = if (data.userState == "CALIBRATING") {
            "— BPM"
        } else {
            "%.1f BPM".format(data.avgHr)
        }

        currentHrText.text = data.hrData.lastOrNull()
            ?.let { "%.1f BPM".format(it.hr) }
            ?: "— BPM"

        val (label, color) = when (data.userState) {
            "ELEVATED"    -> "ELEVATED" to Color.parseColor("#F44336")
            "CALIBRATING" -> "CALIBRATING…" to Color.parseColor("#9E9E9E")
            else          -> "NORMAL" to Color.parseColor("#4CAF50")
        }

        stressLevelText.text = label
        stressCard.setCardBackgroundColor(color)

        lastUpdatedText.text = "Last updated: ${timeFormat.format(Date())}"
    }
}
