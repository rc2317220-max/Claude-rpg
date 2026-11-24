package example;
import org.sunrise.bridge.NativeHost;
import org.sunrise.bridge.SunriseMod;

public final class HealingMod implements SunriseMod {
    public int onServerTick(long tick, int health, int maxHealth) {
        if (NativeHost.protocolVersion() != 1) throw new IllegalStateException("JNI callback failed");
        if (tick == 60) return 999; // intentionally invalid request
        if (tick == 80) return -3; // intentionally invalid request
        return tick % 20 == 0 && health < maxHealth ? 2 : 0;
    }
}
