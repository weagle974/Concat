// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer Abdullah and Concat contributors

package org.standalones.concat;

import android.Manifest;
import android.app.Activity;
import android.app.Fragment;
import android.content.ClipData;
import android.content.ContentResolver;
import android.content.ContentValues;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.database.Cursor;
import android.media.MediaCodec;
import android.media.MediaCodecInfo;
import android.media.MediaCodecList;
import android.media.MediaFormat;
import android.net.Uri;
import android.os.Build;
import android.os.Environment;
import android.provider.MediaStore;
import android.provider.OpenableColumns;
import android.util.Log;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.ArrayList;
import java.util.List;

/**
 * The system's document picker, for an activity that has no Java of its
 * own to receive its answer.
 *
 * The window is a NativeActivity: the framework's own class, with nothing
 * of ours to override, and a picker's result comes back only through an
 * activity's or a fragment's onActivityResult. So this is a fragment - a
 * headless one, added to the activity for the length of one pick - and
 * the result comes back here.
 *
 * What is picked is a content URI, which the engine's decoder cannot open:
 * it reads files by path. Each is copied into the app's own external
 * files, under Imported/, keeping its display name, and the paths are
 * what the engine is handed - through the native method below, which the
 * Rust side registers before it asks for a pick.
 *
 * The way out is here as well: a finished export is written into the
 * app's own folder, which on a phone nothing but the app can open, so
 * {@link #publishVideo} moves it into the phone's Movies through the
 * media store, where the gallery and every file manager show it.
 *
 * And the microphone, for a voiceover: a permission granted while the
 * app runs, asked for through the activity by {@link #askMicrophone}.
 *
 * Compiled by build.rs with the SDK's javac and d8, and loaded at run time
 * from the dex inside the binary; see src/lib.rs.
 */
public class ConcatFiles extends Fragment {
    private static final int REQUEST_PICK = 0xC0C4;
    private static final int REQUEST_MICROPHONE = 0xC0C5;
    private static final String TAG = "concat-files";

    /**
     * Where a finished export goes, under the phone's storage: Movies/Concat.
     * Null before Android 10, which has no media store an app can write
     * through without a permission; there the file stays in the app's folder.
     */
    public static String publishedFolder() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            return null;
        }
        return Environment.DIRECTORY_MOVIES + "/Concat";
    }

    /**
     * Moves the video at {@code path} into {@link #publishedFolder()} through
     * the media store and deletes the original. Two strings come back: the
     * file's path under the phone's storage and null, or null and why it
     * could not be moved - in which case the original is still where it was.
     * Blocking: a video is big and the copy takes a while, so this is called
     * off the window's thread.
     */
    public static String[] publishVideo(Activity activity, String path) {
        String folder = publishedFolder();
        if (folder == null) {
            return new String[] {null, "this phone has no media store to put it in"};
        }
        return publish(activity, path, "video/mp4", folder,
                MediaStore.Video.Media.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY));
    }

    /**
     * {@link #publishVideo} for a small file that is not a video - a
     * subtitle file - of MIME type {@code mime}: moved into the phone's
     * Download/Concat, which every file manager shows, under its own name
     * or "name (1)" where that is taken. The same two strings come back.
     */
    public static String[] publishDocument(Activity activity, String path, String mime) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            return new String[] {null, "this phone has no media store to put it in"};
        }
        return publish(activity, path, mime, Environment.DIRECTORY_DOWNLOADS + "/Concat",
                MediaStore.Downloads.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY));
    }

    /** Moves the file at {@code path} into {@code collection}, under {@code folder}. */
    private static String[] publish(
            Activity activity, String path, String mime, String folder, Uri collection) {
        File file = new File(path);
        ContentResolver resolver = activity.getContentResolver();
        ContentValues values = new ContentValues();
        values.put(MediaStore.MediaColumns.DISPLAY_NAME, file.getName());
        values.put(MediaStore.MediaColumns.MIME_TYPE, mime);
        values.put(MediaStore.MediaColumns.RELATIVE_PATH, folder);
        values.put(MediaStore.MediaColumns.IS_PENDING, 1);
        Uri uri;
        try {
            uri = resolver.insert(collection, values);
        } catch (Exception e) {
            return new String[] {null, "the media store refused the file: " + e.getMessage()};
        }
        if (uri == null) {
            return new String[] {null, "the media store refused the file"};
        }
        InputStream in = null;
        OutputStream out = null;
        try {
            in = new FileInputStream(file);
            out = resolver.openOutputStream(uri);
            if (out == null) {
                throw new IOException("the media store would not open the file for writing");
            }
            byte[] buffer = new byte[1 << 16];
            int read;
            while ((read = in.read(buffer)) > 0) {
                out.write(buffer, 0, read);
            }
            out.close();
            out = null;
            values.clear();
            values.put(MediaStore.MediaColumns.IS_PENDING, 0);
            resolver.update(uri, values, null, null);
        } catch (Exception e) {
            try {
                resolver.delete(uri, null, null);
            } catch (Exception ignored) {
            }
            return new String[] {null, e.getMessage() == null ? e.toString() : e.getMessage()};
        } finally {
            try {
                if (in != null) in.close();
            } catch (IOException ignored) {
            }
            try {
                if (out != null) out.close();
            } catch (IOException ignored) {
            }
        }
        file.delete();
        // The store may have renamed it: a second "name.mp4" is "name (1).mp4".
        return new String[] {folder + "/" + displayName(activity, uri), null};
    }

    /**
     * Whether the app may record from the microphone. When it may not yet,
     * the system's question is put up and false comes back at once; the
     * answer is read the next time this is asked, which is the voiceover
     * panel's Try again. From any thread.
     */
    public static boolean askMicrophone(final Activity activity) {
        if (Build.VERSION.SDK_INT < 23) {
            return true;
        }
        if (activity.checkSelfPermission(Manifest.permission.RECORD_AUDIO)
                == PackageManager.PERMISSION_GRANTED) {
            return true;
        }
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                activity.requestPermissions(
                        new String[] {Manifest.permission.RECORD_AUDIO}, REQUEST_MICROPHONE);
            }
        });
        return false;
    }

    /**
     * Opens a web address in the browser, or in whichever app claims it -
     * the launch screen's guides, the repository, the Discord invite. Null
     * when it went, or why it could not. Asked straight rather than through
     * resolveActivity, which from Android 11 answers no for any app the
     * manifest does not name in its queries. From any thread.
     */
    public static String openLink(Activity activity, String url) {
        try {
            Intent intent = new Intent(Intent.ACTION_VIEW, Uri.parse(url));
            intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
            activity.startActivity(intent);
            return null;
        } catch (android.content.ActivityNotFoundException e) {
            return "no app on this phone opens " + url;
        } catch (RuntimeException e) {
            Log.w(TAG, "could not open " + url, e);
            return String.valueOf(e.getMessage());
        }
    }

    /**
     * What this phone's video encoders are and how they want their frames:
     * a plain-text report, read only. It names the phone and its chip, lists
     * every H.264 and HEVC encoder with the input formats it takes, and for
     * each hardware one configures a codec (and releases it at once, never
     * starting it) to read back the input layout it asks for - the row
     * stride and slice height a ByteBuffer frame has to be written with.
     * Nothing is written to the phone and no setting is touched.
     */
    public static String encoderReport() {
        StringBuilder out = new StringBuilder();
        try {
            report(out);
        } catch (Throwable t) {
            out.append("\nreport stopped: ").append(t).append('\n');
        }
        return out.toString();
    }

    private static void report(StringBuilder out) {
        out.append("phone: ").append(Build.MANUFACTURER).append(' ').append(Build.MODEL)
                .append(" (").append(Build.DEVICE).append(")\n");
        out.append("android: ").append(Build.VERSION.RELEASE).append(", API ")
                .append(Build.VERSION.SDK_INT).append('\n');
        out.append("board: ").append(Build.BOARD).append(" / ").append(Build.HARDWARE).append('\n');
        if (Build.VERSION.SDK_INT >= 31) {
            out.append("chip: ").append(Build.SOC_MANUFACTURER).append(' ')
                    .append(Build.SOC_MODEL).append('\n');
        }
        MediaCodecInfo[] infos = new MediaCodecList(MediaCodecList.REGULAR_CODECS).getCodecInfos();
        for (MediaCodecInfo info : infos) {
            if (!info.isEncoder()) {
                continue;
            }
            for (String type : info.getSupportedTypes()) {
                if (type.equals("video/avc") || type.equals("video/hevc")) {
                    try {
                        describe(out, info, type);
                    } catch (Throwable t) {
                        out.append("  could not read: ").append(t).append('\n');
                    }
                }
            }
        }
    }

    private static void describe(StringBuilder out, MediaCodecInfo info, String type) {
        MediaCodecInfo.CodecCapabilities caps = info.getCapabilitiesForType(type);
        boolean hardware = true;
        if (Build.VERSION.SDK_INT >= 29) {
            hardware = info.isHardwareAccelerated() && !info.isSoftwareOnly();
        }
        out.append('\n').append(info.getName()).append("  ").append(type)
                .append(hardware ? "  HARDWARE" : "  software").append('\n');
        out.append("  takes:");
        boolean nv12 = false;
        boolean p010 = false;
        for (int format : caps.colorFormats) {
            out.append(' ').append(colorName(format));
            nv12 |= format == 21;
            p010 |= format == 54;
        }
        out.append('\n');
        if (type.equals("video/hevc")) {
            boolean main10 = false;
            boolean hdr10 = false;
            for (MediaCodecInfo.CodecProfileLevel level : caps.profileLevels) {
                main10 |= level.profile == 2;
                hdr10 |= level.profile == 4096;
            }
            out.append("  hevc profiles: Main10 ").append(main10)
                    .append(", Main10 HDR10 ").append(hdr10).append('\n');
        }
        MediaCodecInfo.VideoCapabilities video = caps.getVideoCapabilities();
        if (video != null) {
            out.append("  size alignment ").append(video.getWidthAlignment()).append(" x ")
                    .append(video.getHeightAlignment()).append(", 1080x1920 at 60 fps: ")
                    .append(video.areSizeAndRateSupported(1080, 1920, 60.0)).append('\n');
        }
        // Only the encoders that do real work are configured.
        if (!hardware) {
            return;
        }
        int[][] sizes = {{1088, 1920}, {1080, 1920}};
        for (int[] size : sizes) {
            if (nv12) {
                layout(out, info.getName(), type, size[0], size[1], 21);
            }
            if (p010) {
                layout(out, info.getName(), type, size[0], size[1], 54);
            }
        }
    }

    /** Configures the codec for one input format, reads what it asks for, releases it. */
    private static void layout(
            StringBuilder out, String name, String type, int width, int height, int format) {
        MediaCodec codec = null;
        try {
            MediaFormat wanted = MediaFormat.createVideoFormat(type, width, height);
            wanted.setInteger(MediaFormat.KEY_COLOR_FORMAT, format);
            wanted.setInteger(MediaFormat.KEY_BIT_RATE, 9000000);
            wanted.setInteger(MediaFormat.KEY_FRAME_RATE, 60);
            wanted.setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, 1);
            if (format == 54 && type.equals("video/hevc")) {
                wanted.setInteger(MediaFormat.KEY_PROFILE, 2);
            }
            codec = MediaCodec.createByCodecName(name);
            codec.configure(wanted, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE);
            out.append("  ").append(width).append('x').append(height).append(' ')
                    .append(colorName(format)).append(" input: ")
                    .append(codec.getInputFormat()).append('\n');
        } catch (Throwable t) {
            out.append("  ").append(width).append('x').append(height).append(' ')
                    .append(colorName(format)).append(" refused: ").append(t).append('\n');
        } finally {
            if (codec != null) {
                try {
                    codec.release();
                } catch (Throwable ignored) {
                }
            }
        }
    }

    private static String colorName(int format) {
        switch (format) {
            case 19: return "I420";
            case 20: return "I420-packed";
            case 21: return "NV12";
            case 39: return "NV12-packed";
            case 54: return "P010";
            case 0x7F000789: return "Surface";
            case 0x7F420888: return "Flexible";
            default: return "0x" + Integer.toHexString(format);
        }
    }

    /** Registered from Rust: the picked files' paths, or none. */
    public static native void filesPicked(String[] paths);

    /**
     * Shows the picker, offering the MIME types in {@code types} - comma
     * separated, "video/*,audio/*" - or every file when it is empty. From
     * any thread; the fragment is added on the UI thread.
     */
    public static void pick(final Activity activity, final String types) {
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                ConcatFiles fragment = new ConcatFiles();
                activity.getFragmentManager()
                        .beginTransaction()
                        .add(fragment, TAG)
                        .commitAllowingStateLoss();
                activity.getFragmentManager().executePendingTransactions();

                Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
                intent.addCategory(Intent.CATEGORY_OPENABLE);
                intent.setType("*/*");
                if (types != null && !types.isEmpty()) {
                    intent.putExtra(Intent.EXTRA_MIME_TYPES, types.split(","));
                }
                intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, true);
                try {
                    fragment.startActivityForResult(intent, REQUEST_PICK);
                } catch (Exception e) {
                    fragment.done(new ArrayList<Uri>());
                }
            }
        });
    }

    @Override
    public void onActivityResult(int request, int result, Intent data) {
        if (request != REQUEST_PICK) {
            return;
        }
        List<Uri> uris = new ArrayList<Uri>();
        if (result == Activity.RESULT_OK && data != null) {
            ClipData clip = data.getClipData();
            if (clip != null) {
                for (int i = 0; i < clip.getItemCount(); i++) {
                    Uri uri = clip.getItemAt(i).getUri();
                    if (uri != null) {
                        uris.add(uri);
                    }
                }
            } else if (data.getData() != null) {
                uris.add(data.getData());
            }
        }
        done(uris);
    }

    /** Copies what was picked and reports it; then the fragment goes. */
    private void done(final List<Uri> uris) {
        final Activity activity = getActivity();
        try {
            getFragmentManager().beginTransaction().remove(this).commitAllowingStateLoss();
        } catch (Exception ignored) {
        }
        if (activity == null || uris.isEmpty()) {
            filesPicked(new String[0]);
            return;
        }
        // Off the UI thread: a video is big and the copy takes a while.
        new Thread(new Runnable() {
            @Override
            public void run() {
                List<String> paths = new ArrayList<String>();
                File dir = new File(activity.getExternalFilesDir(null), "Imported");
                dir.mkdirs();
                for (Uri uri : uris) {
                    File out = unique(dir, displayName(activity, uri));
                    if (copy(activity, uri, out)) {
                        paths.add(out.getAbsolutePath());
                    }
                }
                filesPicked(paths.toArray(new String[0]));
            }
        }, TAG).start();
    }

    /** The name the picker showed for the file, or one made from the URI. */
    private static String displayName(Activity activity, Uri uri) {
        String name = null;
        Cursor cursor = null;
        try {
            cursor = activity.getContentResolver().query(uri, null, null, null, null);
            if (cursor != null && cursor.moveToFirst()) {
                int column = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME);
                if (column >= 0) {
                    name = cursor.getString(column);
                }
            }
        } catch (Exception ignored) {
        } finally {
            if (cursor != null) {
                cursor.close();
            }
        }
        if (name == null || name.isEmpty()) {
            name = uri.getLastPathSegment();
        }
        if (name == null || name.isEmpty()) {
            name = "import";
        }
        // Whatever the provider called it, it is one file name.
        return name.replace('/', '_').replace('\\', '_');
    }

    /** `name`, or `name (2)`, `name (3)`... until nothing is overwritten. */
    private static File unique(File dir, String name) {
        File out = new File(dir, name);
        if (!out.exists()) {
            return out;
        }
        int dot = name.lastIndexOf('.');
        String stem = dot > 0 ? name.substring(0, dot) : name;
        String ext = dot > 0 ? name.substring(dot) : "";
        for (int n = 2; ; n++) {
            out = new File(dir, stem + " (" + n + ")" + ext);
            if (!out.exists()) {
                return out;
            }
        }
    }

    private static boolean copy(Activity activity, Uri uri, File out) {
        InputStream in = null;
        OutputStream os = null;
        try {
            in = activity.getContentResolver().openInputStream(uri);
            if (in == null) {
                return false;
            }
            os = new FileOutputStream(out);
            byte[] buffer = new byte[1 << 16];
            int read;
            while ((read = in.read(buffer)) > 0) {
                os.write(buffer, 0, read);
            }
            return true;
        } catch (IOException | RuntimeException e) {
            // A provider that revokes its grant mid-copy throws a
            // SecurityException, not an IOException; uncaught on this
            // thread it closed the app and lost the rest of the pick.
            Log.w(TAG, "could not copy " + uri + ": " + e);
            out.delete();
            return false;
        } finally {
            try {
                if (in != null) in.close();
            } catch (IOException ignored) {
            }
            try {
                if (os != null) os.close();
            } catch (IOException ignored) {
            }
        }
    }
}
