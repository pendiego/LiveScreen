package io.github.pendiego.livescreen

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.os.IBinder

class HostService : Service() {
    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START -> start(intent)
            ACTION_STOP -> Hosting.stop(applicationContext)
        }
        return START_NOT_STICKY
    }

    private fun start(intent: Intent) {
        val nm = getSystemService(NotificationManager::class.java)
        nm.createNotificationChannel(NotificationChannel(CHANNEL, "Transmissão", NotificationManager.IMPORTANCE_LOW))
        val open = PendingIntent.getActivity(
            this, 0, Intent(this, MainActivity::class.java), PendingIntent.FLAG_IMMUTABLE
        )
        val stop = PendingIntent.getService(
            this, 1, Intent(this, HostService::class.java).setAction(ACTION_STOP), PendingIntent.FLAG_IMMUTABLE
        )
        val notification = Notification.Builder(this, CHANNEL)
            .setSmallIcon(R.drawable.ic_cast)
            .setContentTitle("LiveScreen transmitindo")
            .setOngoing(true)
            .setContentIntent(open)
            .addAction(Notification.Action.Builder(null, "Parar", stop).build())
            .build()
        if (Build.VERSION.SDK_INT >= 29) {
            startForeground(1, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PROJECTION)
        } else {
            startForeground(1, notification)
        }

        val data = if (Build.VERSION.SDK_INT >= 33) {
            intent.getParcelableExtra("data", Intent::class.java)
        } else {
            @Suppress("DEPRECATION") intent.getParcelableExtra("data")
        }
        if (data == null) {
            Hosting.fail(applicationContext, "captura de tela não autorizada")
            return
        }
        val projection = getSystemService(MediaProjectionManager::class.java)
            .getMediaProjection(intent.getIntExtra("code", 0), data)
        Thread { Hosting.start(applicationContext, projection) }.start()
    }

    override fun onDestroy() {
        Hosting.stop(applicationContext)
    }

    companion object {
        private const val CHANNEL = "livescreen_host"
        private const val ACTION_START = "io.github.pendiego.livescreen.START"
        private const val ACTION_STOP = "io.github.pendiego.livescreen.STOP"

        fun startIntent(ctx: Context, code: Int, data: Intent): Intent =
            Intent(ctx, HostService::class.java).setAction(ACTION_START).putExtra("code", code).putExtra("data", data)
    }
}
