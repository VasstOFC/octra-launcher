# Octra App

Minecraft launcher based on [Modrinth Theseus](https://github.com/modrinth/code). Catalog still uses Modrinth’s API.

**Downloads / auto-update:** [Releases](https://github.com/VasstOFC/octra-launcher/releases)

```powershell
pnpm install
pnpm app:dev
```

Windows installer: **Actions → Octra App update** on this repo (branch `app`) builds `Octra-setup.exe`. Needs GitHub secret `TAURI_SIGNING_PRIVATE_KEY`. After you rename the repo to `octra-app`, old `octra-launcher` download URLs keep working via GitHub redirect.

## Upstream (Modrinth monorepo)

## Development

This repository contains two primary packages. For detailed development information, please refer to their respective guides:

- [Website frontend](https://docs.modrinth.com/contributing/knossos/)
- [Desktop app](https://docs.modrinth.com/contributing/theseus/)

## Contributing

We welcome contributions! Before submitting any contributions, please read our [contributing guidelines](https://docs.modrinth.com/contributing/getting-started/).

If you plan to fork this repository for your own purposes, please review our [copying guidelines](COPYING.md).

## Security

If you discover a security vulnerability within our codebase, please follow our [responsible disclosure guidelines](https://modrinth.com/legal/security).

## Support

If you need help with the Modrinth web interface or app, please visit our [support page](https://support.modrinth.com). For general inquiries, you can also join our [Discord server](https://discord.modrinth.com).

## License

All packages in this repository are licensed under their respective licenses. Refer to the LICENSE file in each package for more information.
