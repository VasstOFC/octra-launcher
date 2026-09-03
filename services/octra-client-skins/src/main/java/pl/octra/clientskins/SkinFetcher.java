package pl.octra.clientskins;

import java.net.HttpURLConnection;
import java.net.URL;
import java.util.Locale;

public final class SkinFetcher {
	private static final String DEFAULT_ROOT = "http://92.5.186.6";

	private SkinFetcher() {}

	public static String apiRoot() {
		String fromProp = System.getProperty("octra.skins.url", "").trim();
		if (!fromProp.isEmpty()) {
			return fromProp.replaceAll("/+$", "");
		}
		String fromEnv = System.getenv("OCTRA_SKINS_URL");
		if (fromEnv != null && !fromEnv.isBlank()) {
			return fromEnv.trim().replaceAll("/+$", "");
		}
		return DEFAULT_ROOT;
	}

	public static FetchedSkin download(String playerName) {
		HttpURLConnection connection = null;
		try {
			URL url = new URL(apiRoot() + "/skins/MinecraftSkins/" + playerName + ".png");
			connection = (HttpURLConnection) url.openConnection();
			connection.setConnectTimeout(8000);
			connection.setReadTimeout(8000);
			connection.setRequestMethod("GET");
			connection.setInstanceFollowRedirects(true);
			connection.setRequestProperty("User-Agent", "OctraClientSkins/1.0");
			int code = connection.getResponseCode();
			if (code != 200) {
				return null;
			}
			String model = connection.getHeaderField("X-Lumen-Model");
			byte[] png = connection.getInputStream().readAllBytes();
			if (png.length < 64) {
				return null;
			}
			return new FetchedSkin(png, isSlim(model));
		} catch (Exception e) {
			OctraClientSkins.LOGGER.debug("Skin fetch failed for {}: {}", playerName, e.toString());
			return null;
		} finally {
			if (connection != null) {
				connection.disconnect();
			}
		}
	}

	private static boolean isSlim(String model) {
		if (model == null) {
			return false;
		}
		String normalized = model.trim().toLowerCase(Locale.ROOT);
		return "slim".equals(normalized) || "alex".equals(normalized);
	}

	public record FetchedSkin(byte[] png, boolean slim) {}
}
