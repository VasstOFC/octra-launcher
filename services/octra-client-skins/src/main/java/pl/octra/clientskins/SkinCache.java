package pl.octra.clientskins;

import com.mojang.authlib.GameProfile;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.NativeImageBackedTexture;
import net.minecraft.util.Identifier;

import java.io.ByteArrayInputStream;
import java.util.Locale;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * Replaces player skins with Octra registry PNGs when present.
 * Falls through to vanilla (authlib / Mojang) when the registry has no file.
 */
public final class SkinCache {
	private static final long MISS_TTL_MS = 5L * 60L * 1000L;
	private static final ExecutorService WORKER = Executors.newFixedThreadPool(2, runnable -> {
		Thread thread = new Thread(runnable, "octra-client-skins");
		thread.setDaemon(true);
		return thread;
	});
	private static final ConcurrentHashMap<String, Entry> CACHE = new ConcurrentHashMap<>();

	private SkinCache() {}

	public static Identifier texture(GameProfile profile, Identifier vanilla) {
		Entry entry = lookup(profile);
		if (entry != null && entry.status == Status.READY && entry.texture != null) {
			return entry.texture;
		}
		return vanilla;
	}

	public static String model(GameProfile profile, String vanilla) {
		Entry entry = lookup(profile);
		if (entry != null && entry.status == Status.READY && entry.model != null) {
			return entry.model;
		}
		return vanilla;
	}

	private static Entry lookup(GameProfile profile) {
		if (profile == null) {
			return null;
		}
		String name = profile.getName();
		if (name == null || name.isBlank() || name.length() > 16) {
			return null;
		}
		String key = name.toLowerCase(Locale.ROOT);
		Entry entry = CACHE.get(key);
		if (entry == null) {
			Entry pending = new Entry(Status.PENDING, null, null, 0);
			if (CACHE.putIfAbsent(key, pending) == null) {
				WORKER.execute(() -> fetchAndUpload(key, name));
			}
			return pending;
		}
		if (entry.status == Status.MISS && System.currentTimeMillis() >= entry.missUntilMs) {
			CACHE.put(key, new Entry(Status.PENDING, null, null, 0));
			WORKER.execute(() -> fetchAndUpload(key, name));
		}
		return entry;
	}

	private static void fetchAndUpload(String key, String name) {
		SkinFetcher.FetchedSkin fetched = SkinFetcher.download(name);
		if (fetched == null) {
			CACHE.put(key, new Entry(Status.MISS, null, null, System.currentTimeMillis() + MISS_TTL_MS));
			return;
		}
		MinecraftClient client = MinecraftClient.getInstance();
		if (client == null) {
			CACHE.put(key, new Entry(Status.MISS, null, null, System.currentTimeMillis() + MISS_TTL_MS));
			return;
		}
		client.execute(() -> {
			try {
				NativeImage image = NativeImage.read(new ByteArrayInputStream(fetched.png()));
				Identifier id = new Identifier(OctraClientSkins.MOD_ID, "dyn/" + key);
				client.getTextureManager().registerTexture(id, new NativeImageBackedTexture(image));
				String model = fetched.slim() ? "slim" : "default";
				CACHE.put(key, new Entry(Status.READY, id, model, 0));
				OctraClientSkins.LOGGER.info("Loaded Octra skin for {}", name);
			} catch (Exception e) {
				OctraClientSkins.LOGGER.warn("Failed to upload skin for {}: {}", name, e.toString());
				CACHE.put(key, new Entry(Status.MISS, null, null, System.currentTimeMillis() + MISS_TTL_MS));
			}
		});
	}

	private enum Status {
		PENDING,
		READY,
		MISS
	}

	private record Entry(Status status, Identifier texture, String model, long missUntilMs) {}
}
