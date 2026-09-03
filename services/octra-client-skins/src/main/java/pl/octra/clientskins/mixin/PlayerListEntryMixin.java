package pl.octra.clientskins.mixin;

import com.mojang.authlib.GameProfile;
import net.minecraft.client.network.PlayerListEntry;
import net.minecraft.util.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pl.octra.clientskins.SkinCache;

@Mixin(PlayerListEntry.class)
public abstract class PlayerListEntryMixin {
	@Shadow
	public abstract GameProfile getProfile();

	@Inject(method = "getSkinTexture", at = @At("RETURN"), cancellable = true)
	private void octraTexture(CallbackInfoReturnable<Identifier> cir) {
		cir.setReturnValue(SkinCache.texture(getProfile(), cir.getReturnValue()));
	}

	@Inject(method = "getModel", at = @At("RETURN"), cancellable = true)
	private void octraModel(CallbackInfoReturnable<String> cir) {
		cir.setReturnValue(SkinCache.model(getProfile(), cir.getReturnValue()));
	}
}
