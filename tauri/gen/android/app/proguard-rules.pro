# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more on customizing ProGuard/R8 rules, see:
#  https://r8.googlesource.com/r8/+/refs/heads/main/doc/keepanno-doc.md
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# ML Kit OCR classes are resolved reflectively by tauri-plugin-device-ai-apis
# (DeviceAiPlugin.kt::createScriptRecognizer):
#   1. Class.forName("com.google.mlkit.vision.text.japanese.JapaneseTextRecognizerOptions")
#   2. getMethod("Builder") and getMethod("build") on that class hierarchy
#   3. TextRecognition.class.methods.first { name == "getClient" && parameterCount == 1 }
#      — base class com.google.mlkit.vision.text.TextRecognition, outside the
#      japanese package
# R8 sees no static references to these string-resolved names and would
# strip/rename them in release builds (isMinifyEnabled=true), silently
# degrading native Japanese OCR to the WASM fallback.
-keep class com.google.mlkit.vision.text.japanese.** { *; }
-keep class com.google.mlkit.vision.text.TextRecognition { *; }

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