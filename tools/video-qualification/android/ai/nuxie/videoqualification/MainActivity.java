package ai.nuxie.videoqualification;
import android.app.Activity;
import android.os.Bundle;
import android.os.Handler;
import android.widget.TextView;
import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
public final class MainActivity extends Activity {
  static { System.loadLibrary("video_qualification"); }
  private native int open(String source, boolean embedded, boolean sync, boolean benchmark);
  private native int tick();
  private native String caption();
  private native String metrics();
  private native byte[] preview();
  private native void close();
  private native boolean lifecycleReady();
  private native void lifecyclePause(boolean suspended);
  private native void lifecycleSample();
  private boolean lifecycleRequested, lifecyclePaused, lifecycleVerified;
  private final Runnable sampleWhilePaused = new Runnable() {
    public void run() {
      if (!lifecyclePaused || stopped) return;
      try { lifecycleSample(); handler.postDelayed(this, 50); }
      catch (Throwable error) { finishProof("FAIL: " + error); }
    }
  };
  private final Handler handler = new Handler();
  private TextView label;
  private TextView captions;
  private android.widget.ImageView videoPreview;
  private boolean redCaption, blueCaption;
  private boolean stopped;
  @Override
  public void onCreate(Bundle state) {
    super.onCreate(state);
    label = new TextView(this);
    label.setText("Nuxie Android video proof running");
    captions = new TextView(this);
    captions.setAccessibilityLiveRegion(
        android.view.View.ACCESSIBILITY_LIVE_REGION_POLITE);
    android.widget.LinearLayout layout = new android.widget.LinearLayout(this);
    layout.setOrientation(android.widget.LinearLayout.VERTICAL);
    layout.addView(label);
    videoPreview = new android.widget.ImageView(this);
    videoPreview.setContentDescription("Verified runtime video with green UI overlay");
    layout.addView(videoPreview);
    layout.addView(captions);
    setContentView(layout);
    try {
      new File(getFilesDir(), "video-proof-result.txt").delete();
      String fixture = getIntent().getBooleanExtra("benchmark", false) ? "red-blue-720p.mp4" : getIntent().getBooleanExtra("sync", false)
                           ? "red-blue-sync.mp4"
                           : "red-blue-audio.mp4";
      File file = new File(getCacheDir(), fixture);
      try (InputStream input = getAssets().open(fixture);
           FileOutputStream output = new FileOutputStream(file)) {
        byte[] bytes = new byte[8192];
        int count;
        while ((count = input.read(bytes)) != -1)
          output.write(bytes, 0, count);
      }
      if (getIntent().getBooleanExtra("teardown", false)) {
        runTeardownProof(file);
        return;
      }
      if (getIntent().getBooleanExtra("focus", false)) {
        runFocusProof(file);
        return;
      }
      open(file.getAbsolutePath(),
           getIntent().getBooleanExtra("embedded", false),
           getIntent().getBooleanExtra("sync", false),
           getIntent().getBooleanExtra("benchmark", false));
      handler.post(this::advance);
    } catch (Throwable error) {
      finishProof("FAIL: " + error);
    }
  }
  private void runTeardownProof(File source) {
    new Thread(() -> {
      java.util.concurrent.CountDownLatch release = new java.util.concurrent.CountDownLatch(1);
      ai.nuxie.runtime.VideoPlayer player = null;
      String result;
      try {
        player = new ai.nuxie.runtime.VideoPlayer(this, source.getAbsolutePath(), 0, 1024 * 1024, 0);
        java.lang.reflect.Field handlerField = player.getClass().getDeclaredField("handler");
        java.lang.reflect.Field threadField = player.getClass().getDeclaredField("thread");
        handlerField.setAccessible(true);
        threadField.setAccessible(true);
        Handler workerHandler = (Handler)handlerField.get(player);
        Thread worker = (Thread)threadField.get(player);
        java.util.concurrent.CountDownLatch entered = new java.util.concurrent.CountDownLatch(1);
        if (!workerHandler.post(() -> {
          entered.countDown();
          try { release.await(); }
          catch (InterruptedException error) { Thread.currentThread().interrupt(); }
        })) throw new IllegalStateException("worker rejected blocker");
        if (!entered.await(5, java.util.concurrent.TimeUnit.SECONDS))
          throw new IllegalStateException("worker never entered blocker");
        boolean timedOut = false;
        try { player.close(); }
        catch (IllegalStateException error) {
          if (!"video decoder teardown timed out".equals(error.getMessage())) throw error;
          timedOut = true;
        }
        if (!timedOut || !worker.isAlive())
          throw new IllegalStateException("first close must time out with worker alive");
        handler.postDelayed(release::countDown, 300);
        player.close();
        if (worker.isAlive()) throw new IllegalStateException("retry acknowledged a live decoder worker");
        player.close();
        result = "PASS: blocked decoder teardown timed out; retry waited for worker termination; repeated close is idempotent";
      } catch (Throwable error) {
        result = "FAIL: " + error;
      } finally {
        release.countDown();
        if (player != null) {
          try { player.close(); } catch (Throwable ignored) {}
        }
      }
      String completed = result;
      handler.post(() -> finishProof(completed));
    }, "video-teardown-proof").start();
  }
  private ai.nuxie.runtime.VideoPlayer focusPlayer;
  private android.media.AudioManager focusAudio;
  private final android.media.AudioManager.OnAudioFocusChangeListener challenger = change -> {};
  private int focusStage;
  private long focusDeadline, focusLostAt;
  @SuppressWarnings("deprecation")
  private void runFocusProof(File source) {
    focusAudio = (android.media.AudioManager)getSystemService(AUDIO_SERVICE);
    focusPlayer = new ai.nuxie.runtime.VideoPlayer(this, source.getAbsolutePath(), 0, 1024 * 1024, 0);
    focusPlayer.action(4, 0.2, 0);
    focusStage = -1;
    focusDeadline = android.os.SystemClock.elapsedRealtime() + 10000;
    handler.post(new Runnable() {
      public void run() {
        if (stopped) return;
        try {
          if (focusPlayer.failure() != null) throw new IllegalStateException(focusPlayer.failure());
          long now = android.os.SystemClock.elapsedRealtime();
          if (now > focusDeadline) throw new IllegalStateException("audio focus timeout stage=" + focusStage);
          if (focusStage == -1 && focusPlayer.ready()) {
            focusPlayer.action(0, 0, 0);
            focusStage = 0;
          } else if (focusStage == 0 && focusPlayer.playing()) {
            int result = focusAudio.requestAudioFocus(challenger, android.media.AudioManager.STREAM_MUSIC,
                android.media.AudioManager.AUDIOFOCUS_GAIN_TRANSIENT);
            if (result != android.media.AudioManager.AUDIOFOCUS_REQUEST_GRANTED)
              throw new IllegalStateException("challenger focus denied");
            focusStage = 1;
          } else if (focusStage == 1 && focusPlayer.interrupted() && !focusPlayer.playing()) {
            // The runtime delivers Pause while retaining requested playback.
            focusPlayer.action(1, 0, 0);
            // An author command must not bypass an active native interruption.
            focusPlayer.action(0, 0, 0);
            focusLostAt = now;
            focusStage = 2;
          } else if (focusStage == 2 && now - focusLostAt >= 150) {
            if (focusPlayer.playing()) throw new IllegalStateException("Play bypassed native interruption");
            focusAudio.abandonAudioFocus(challenger);
            focusStage = 3;
          } else if (focusStage == 3 && focusPlayer.takeInterruptionEnded()) {
            if (focusPlayer.interrupted() || focusPlayer.playing())
              throw new IllegalStateException("focus gain bypassed runtime intent");
            focusPlayer.action(0, 0, 0);
            focusStage = 4;
          } else if (focusStage == 4 && focusPlayer.playing()) {
            finishProof("PASS: real Android transient audio focus loss, paused native player, retained focus request, consumed recovery edge, explicit resume, teardown");
            return;
          }
          handler.postDelayed(this, 20);
        } catch (Throwable error) { finishProof("FAIL: " + error); }
      }
    });
  }
  private void closeFocusProof() {
    if (focusAudio != null) focusAudio.abandonAudioFocus(challenger);
    if (focusPlayer != null) { focusPlayer.close(); focusPlayer = null; }
  }
  private void advance() {
    if (stopped || lifecyclePaused)
      return;
    try {
      int status = tick();
      if (getIntent().getBooleanExtra("benchmark", false)) {
        byte[] rgba = preview();
        if (rgba != null && rgba.length == 64 * 32 * 4) {
          int[] colors = new int[64 * 32];
          for (int i = 0; i < colors.length; i++) {
            int offset = i * 4;
            colors[i] = ((rgba[offset + 3] & 255) << 24) | ((rgba[offset] & 255) << 16)
                | ((rgba[offset + 1] & 255) << 8) | (rgba[offset + 2] & 255);
          }
          videoPreview.setImageBitmap(android.graphics.Bitmap.createBitmap(colors, 64, 32, android.graphics.Bitmap.Config.ARGB_8888));
        }
      }
      if (getIntent().getBooleanExtra("lifecycle", false) && !lifecycleRequested && lifecycleReady()) {
        lifecycleRequested = true;
        if (!moveTaskToBack(true)) throw new IllegalStateException("cannot background task");
        return;
      }
      String text = caption();
      if (text != null && !text.contentEquals(captions.getText()))
        captions.setText(text);
      redCaption |= "Red scene".equals(text);
      blueCaption |= "Blue scene".equals(text);
      if (status == 1 && getIntent().getBooleanExtra("benchmark", false)) {
        finishProof("PASS: 720p Vulkan benchmark; " + metrics());
        return;
      }
      if (status == 1) {
        if (!getIntent().getBooleanExtra("sync", false) &&
            (!redCaption || !blueCaption))
          throw new IllegalStateException("missing visible captions");
        finishProof("PASS: MediaPlayer -> .nux -> Vulkan; pixels, captions; " +
                    (getIntent().getBooleanExtra("sync", false)
                         ? "synchronized players, disposal; "
                         : "seek, two loops, reclamation/reopen, completion, "
                               + "disposal; ") +
                    "muted AAC; platform-selected decoder; embedded=" +
                    getIntent().getBooleanExtra("embedded", false) + "; " +
                    metrics() + "; OS background/resume=" + lifecycleVerified);
      } else
        handler.postDelayed(this::advance, 16);
    } catch (Throwable error) {
      String detail = "";
      if (getIntent().getBooleanExtra("benchmark", false)) {
        try { detail = "; " + metrics(); } catch (Throwable ignored) {}
      }
      finishProof("FAIL: " + error + detail);
    }
  }
  private void finishProof(String result) {
    closeFocusProof();
    stopped = true;
    label.setText(result);
    android.util.Log.i("NuxVideoProof", result);
    try (FileOutputStream output = new FileOutputStream(
             new File(getFilesDir(), "video-proof-result.txt"))) {
      output.write(result.getBytes(StandardCharsets.UTF_8));
    } catch (Exception ignored) {
    }
    close();
  }
  @Override
  public void onPause() {
    if (getIntent().getBooleanExtra("benchmark", false) && !stopped)
      finishProof("FAIL: 720p benchmark interrupted by app suspension");
    if (lifecycleRequested && !stopped && !lifecyclePaused) {
      lifecyclePaused = true;
      lifecyclePause(true);
      handler.post(sampleWhilePaused);
    }
    super.onPause();
  }
  @Override
  public void onResume() {
    super.onResume();
    if (lifecyclePaused && !stopped) {
      try {
        handler.removeCallbacks(sampleWhilePaused);
        lifecycleSample();
        lifecyclePause(false);
        lifecyclePaused = false;
        lifecycleVerified = true;
        handler.post(this::advance);
      } catch (Throwable error) { finishProof("FAIL: " + error); }
    }
  }
  @Override
  public void onDestroy() {
    closeFocusProof();
    stopped = true;
    handler.removeCallbacksAndMessages(null);
    close();
    super.onDestroy();
  }
}
