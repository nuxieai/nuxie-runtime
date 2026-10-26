package ai.nuxie.audiovalidation;

import android.app.Activity;
import android.media.AudioAttributes;
import android.media.AudioFormat;
import android.media.AudioManager;
import android.media.AudioTrack;
import android.os.Bundle;
import android.os.Build;
import android.util.Log;
import android.widget.TextView;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.atomic.AtomicBoolean;
import org.json.JSONObject;

/** Device-only adapter: the PCM mixer and renderer are the actual Rust backend. */
public final class MainActivity extends Activity {
    static { System.loadLibrary("audio_device_validation"); }
    private static native long nativeNew(int rate, int channels, int block);
    private static native long nativePull(long probe, ByteBuffer output, int frames);
    private static native void nativeSetPhase(long probe, int phase);
    private static native String nativeReport(long probe);
    private static native void nativeDestroy(long probe);
    private final AtomicBoolean cancelled = new AtomicBoolean();
    private TextView view;

    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        view = new TextView(this);
        view.setText("Running audio validation…");
        setContentView(view);
        new Thread(this::validateAudio, "nuxie-audio-validation").start();
    }

    @Override protected void onDestroy() {
        cancelled.set(true);
        super.onDestroy();
    }

    private void validateAudio() {
        long probe = 0;
        AudioTrack track = null;
        JSONObject result = new JSONObject();
        try {
            final int rate = AudioTrack.getNativeOutputSampleRate(AudioManager.STREAM_MUSIC);
            final int channels = 2;
            final int pullFrames = 512;
            int minBytes = AudioTrack.getMinBufferSize(rate, AudioFormat.CHANNEL_OUT_STEREO,
                    AudioFormat.ENCODING_PCM_FLOAT);
            if (minBytes <= 0) throw new IllegalStateException("unsupported float output: " + minBytes);
            track = new AudioTrack.Builder()
                .setAudioAttributes(new AudioAttributes.Builder()
                    .setUsage(AudioAttributes.USAGE_MEDIA)
                    .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC).build())
                .setAudioFormat(new AudioFormat.Builder().setSampleRate(rate)
                    .setChannelMask(AudioFormat.CHANNEL_OUT_STEREO)
                    .setEncoding(AudioFormat.ENCODING_PCM_FLOAT).build())
                .setTransferMode(AudioTrack.MODE_STREAM)
                .setBufferSizeInBytes(Math.max(minBytes * 2, pullFrames * channels * 4)).build();
            if (track.getState() != AudioTrack.STATE_INITIALIZED) throw new IllegalStateException("AudioTrack not initialized");
            probe = nativeNew(rate, channels, 256);
            if (probe == 0) throw new IllegalStateException("Rust probe allocation failed");
            ByteBuffer output = ByteBuffer.allocateDirect(pullFrames * channels * 4).order(ByteOrder.nativeOrder());
            long writtenFrames = 0;
            boolean started = false;
            for (int phase = 0; phase < 4; phase++) {
                nativeSetPhase(probe, phase);
                int phaseFrames = 0;
                while (phaseFrames < rate) {
                    if (cancelled.get()) throw new IllegalStateException("activity cancelled");
                    int frames = Math.min(pullFrames, rate - phaseFrames);
                    long read = nativePull(probe, output, frames);
                    if (read < 0 || read > frames) throw new IllegalStateException("invalid read count " + read);
                    output.position(0);
                    output.limit(frames * channels * 4);
                    while (output.hasRemaining()) {
                        int bytes = track.write(output, output.remaining(), AudioTrack.WRITE_BLOCKING);
                        if (bytes <= 0) throw new IllegalStateException("AudioTrack write failed: " + bytes);
                        if ((bytes % (channels * 4)) != 0) throw new IllegalStateException("unaligned AudioTrack write");
                        writtenFrames += bytes / (channels * 4);
                    }
                    // Prime one complete buffer before asking the device to consume it.
                    if (!started) { track.play(); started = true; }
                    output.clear();
                    phaseFrames += frames;
                }
            }
            int underrunsBeforeDrain = track.getUnderrunCount();
            long deadline = android.os.SystemClock.elapsedRealtime() + 3000;
            while (Integer.toUnsignedLong(track.getPlaybackHeadPosition()) < writtenFrames
                    && android.os.SystemClock.elapsedRealtime() < deadline && !cancelled.get()) {
                Thread.sleep(10);
            }
            long played = Integer.toUnsignedLong(track.getPlaybackHeadPosition());
            if (played < writtenFrames) throw new IllegalStateException("output did not drain: " + played + "/" + writtenFrames);
            JSONObject rust = new JSONObject(nativeReport(probe));
            result.put("rust", rust);
            if (!rust.getBoolean("pass")) throw new IllegalStateException("Rust validation failed: " + rust);
            result.put("platform", "android-emulator");
            result.put("model", Build.MODEL);
            result.put("api", Build.VERSION.SDK_INT);
            result.put("sampleRate", rate);
            result.put("channels", channels);
            result.put("writtenFrames", writtenFrames);
            result.put("playedFrames", played);
            result.put("underrunsBeforeDrain", underrunsBeforeDrain);
            result.put("underruns", track.getUnderrunCount());
            result.put("bufferFrames", track.getBufferSizeInFrames());
            result.put("hostOutputPassed", true);
        } catch (Throwable failure) {
            try { result.put("hostOutputPassed", false); result.put("error", failure.toString()); } catch (Exception ignored) {}
            Log.e("NuxieAudioValidation", "audio validation failed", failure);
        } finally {
            if (track != null) { try { track.stop(); } catch (Exception ignored) {} track.release(); }
            if (probe != 0) nativeDestroy(probe);
            try { result.put("teardownCompleted", true); } catch (Exception ignored) {}
            final String json = result.toString();
            try (java.io.FileOutputStream file = openFileOutput("audio-validation.json", MODE_PRIVATE)) {
                file.write(json.getBytes(StandardCharsets.UTF_8));
            } catch (Exception failure) { Log.e("NuxieAudioValidation", "write report", failure); }
            Log.i("NuxieAudioValidation", "RESULT " + json);
            runOnUiThread(() -> view.setText(json));
        }
    }
}
