package com.example.mobilestressmonitor

import retrofit2.http.GET
import retrofit2.http.Query

interface ApiService {
    @GET("info")
    suspend fun getInfo(@Query("key") key: String): InfoResponse
}
