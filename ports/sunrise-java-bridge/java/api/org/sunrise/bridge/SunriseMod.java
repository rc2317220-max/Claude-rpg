package org.sunrise.bridge;

/** Experimental bridge contract. Not Forge, NeoForge or Fabric. */
public interface SunriseMod {
    /** Requests a bounded heal; Rust is authoritative and validates the result. */
    int onServerTick(long tick, int health, int maxHealth);
}
