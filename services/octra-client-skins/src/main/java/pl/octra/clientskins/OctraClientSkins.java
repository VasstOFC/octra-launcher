package pl.octra.clientskins;

import net.fabricmc.api.ClientModInitializer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class OctraClientSkins implements ClientModInitializer {
	public static final String MOD_ID = "octra-client-skins";
	public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

	@Override
	public void onInitializeClient() {
		LOGGER.info("Octra client skins -> {}", SkinFetcher.apiRoot());
	}
}
