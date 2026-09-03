package pl.octra.clientskins.mixin;

import net.minecraft.client.network.AbstractClientPlayerEntity;
import net.minecraft.util.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pl.octra.clientskins.SkinCache;

@Mixin(AbstractClientPlayerEntity.class)
public abstract class AbstractClientPlayerEntityMixin {
	@Inject(method = "getSkinTexture", at = @At("RETURN"), cancellable = true)
	private void octraTexture(CallbackInfoReturnable<Identifier> cir) {
		AbstractClientPlayerEntity self = (AbstractClientPlayerEntity) (Object) this;
		cir.setReturnValue(SkinCache.texture(self.getGameProfile(), cir.getReturnValue()));
	}

	@Inject(method = "getModel", at = @At("RETURN"), cancellable = true)
	private void octraModel(CallbackInfoReturnable<String> cir) {
		AbstractClientPlayerEntity self = (AbstractClientPlayerEntity) (Object) this;
		cir.setReturnValue(SkinCache.model(self.getGameProfile(), cir.getReturnValue()));
	}
}
