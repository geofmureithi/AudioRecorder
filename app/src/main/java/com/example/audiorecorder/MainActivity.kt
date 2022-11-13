package com.example.audiorecorder

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material.Button
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Surface
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import com.example.audiorecorder.ui.theme.AudioRecorderTheme
import com.example.audiorecorder.lib.AudioSineWaveGen;

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            AudioRecorderTheme {
                // A surface container using the 'background' color from the theme
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colors.background
                ) {
                    Greeting("Android")
                }
            }
        }
    }
    companion object {
        // Used to load the 'audio_lib' library on application startup.
        init {
            System.loadLibrary("android_lib")
        }
    }
}

@Composable
fun Greeting(name: String) {
    Text(text = "Hello $name!")
}

@Composable
fun StreamButton() {
    Button(onClick = {
        AudioSineWaveGen.audioProbe()
    }) {
        Text(text = "Stream")
    }
}

@Preview(showBackground = true)
@Composable
fun DefaultPreview() {
    AudioRecorderTheme {
        Greeting("Android")
        StreamButton()
    }
}