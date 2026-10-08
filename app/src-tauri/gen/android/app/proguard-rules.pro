# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# If your project uses WebView with JS, uncomment the following
# and specify the fully qualified class name to the JavaScript interface
# class:
#-keepclassmembers class fqcn.of.javascript.interface.for.webview {
#   public *;
#}

# Uncomment this to preserve the line number information for
# debugging stack traces.
#-keepattributes SourceFile,LineNumberTable

# If you keep the line number information, uncomment this to
# hide the original source file name.
#-renamesourcefileattribute SourceFile
# Der Tresor wird nur von Rust ueber JNI gerufen (src-tauri/src/tresor.rs).
# R8 sieht keinen Aufrufer und wuerde verschliessen/oeffnen sonst entfernen
# oder umbenennen -- dann scheiterte jede Anmeldung erst im Release-Bau.
-keep class de.openany.app.Tresor {
    public static byte[] verschliessen(byte[]);
    public static byte[] oeffnen(byte[]);
    private static native void anmelden();
}

# Der Wachdienst: Rust ruft zeige/protokoll ueber JNI zurueck
# (src-tauri/src/wachdienst.rs), und die native Seite sucht laufen/anhalten/
# netz/abmelden unter genau diesen Namen. R8 saehe keinen Aufrufer.
-keep class de.openany.app.Wachdienst$Companion { *; }
-keep class de.openany.app.Wachdienst {
    public static void zeige(java.lang.String, java.lang.String);
    public static void protokoll(java.lang.String);
    public static native <methods>;
}
# MainActivity.aufgefrischt ruft Rust ebenfalls zurueck.
-keep class de.openany.app.MainActivity {
    public static void aufgefrischt();
}
# Das Standbild eines Videos: Rust ruft `ziehen` ueber JNI (src-tauri/src/standbild.rs).
-keep class de.openany.app.Standbild {
    public static byte[] ziehen(java.lang.String);
    private static native void anmelden();
}
