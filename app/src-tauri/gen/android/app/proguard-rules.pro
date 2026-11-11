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
    public static byte[] vorschau(java.lang.String);
    private static native void anmelden();
}
# Fotos vom Geraet uebernehmen: Rust fragt die Medienbibliothek ueber JNI
# (src-tauri/src/medien.rs), und der Dienst liest den Stand der Schlange
# (uebernahme.rs) unter genau diesen Namen.
-keep class de.openany.app.Medien {
    public static java.lang.String uebersicht();
    public static java.lang.String liste(java.lang.String, long);
    public static int oeffnen(java.lang.String);
    public static void dienstStarten();
    public static boolean gemessen();
    public static void waechterEinplanen(boolean, boolean, boolean);
    private static native void anmelden();
}
-keep class de.openany.app.Uebernahmedienst {
    public static native <methods>;
}
# Kalender im Telefon (K1): Rust ruft spiegeln/abmelden und wird ueber
# anmelden/laufen gerufen (systemkalender.rs).
-keep class de.openany.app.Systemkalender {
    public static java.lang.String spiegeln(java.lang.String);
    public static void abmelden();
    public static void leeren();
    public static java.lang.String aenderungen();
    public static void bestaetigen(java.lang.String);
    public static native <methods>;
    private static native void anmelden();
}
# Kontakte im Telefon (K4): wie der Kalender (systemkontakte.rs).
-keep class de.openany.app.Systemkontakte {
    public static java.lang.String spiegeln(java.lang.String);
    public static void leeren();
    public static java.lang.String aenderungen();
    public static void bestaetigen(java.lang.String);
    public static native <methods>;
    private static native void anmelden();
}
# Der Fotowaechter (D5): Rust sucht laufen() unter genau diesem Namen.
-keep class de.openany.app.Fotowaechter {
    public static native <methods>;
}
