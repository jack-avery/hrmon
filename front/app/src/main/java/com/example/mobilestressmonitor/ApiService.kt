package com.example.mobilestressmonitor

import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST
import retrofit2.http.Query
import retrofit2.Response

interface ApiService {
    @GET("info")
    suspend fun getInfo(@Query("key") key: String): InfoResponse

    @POST("flush")
    suspend fun flush(@Body request: FlushRequest): Response<Unit>
}
