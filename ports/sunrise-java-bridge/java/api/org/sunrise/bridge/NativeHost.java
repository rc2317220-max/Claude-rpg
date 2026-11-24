package org.sunrise.bridge;

public final class NativeHost {
    private NativeHost() {}
    /** Calls Rust via JNI. */
    public static native int protocolVersion();
}
