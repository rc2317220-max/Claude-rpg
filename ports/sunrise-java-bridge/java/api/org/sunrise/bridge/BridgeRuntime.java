package org.sunrise.bridge;
import java.net.URLClassLoader;
import java.nio.file.Path;

public final class BridgeRuntime implements AutoCloseable {
    private final URLClassLoader loader;
    private final SunriseMod mod;
    private BridgeRuntime(URLClassLoader loader, SunriseMod mod) {
        this.loader = loader;
        this.mod = mod;
    }
    public static BridgeRuntime open(String jar, String entry) throws Exception {
        URLClassLoader loader = new URLClassLoader(
            new java.net.URL[]{Path.of(jar).toUri().toURL()},
            SunriseMod.class.getClassLoader());
        try {
            Class<?> type = Class.forName(entry, false, loader);
            if (!SunriseMod.class.isAssignableFrom(type) || type.getClassLoader() != loader)
                throw new IllegalArgumentException("Expected bridge entry from supplied JAR");
            SunriseMod mod = (SunriseMod)type.getDeclaredConstructor().newInstance();
            return new BridgeRuntime(loader, mod);
        } catch (Exception | LinkageError e) {
            loader.close();
            throw e;
        }
    }
    public int tick(long tick, int hp, int maxHp) {
        return mod.onServerTick(tick, hp, maxHp);
    }
    public void close() throws Exception { loader.close(); }
}
