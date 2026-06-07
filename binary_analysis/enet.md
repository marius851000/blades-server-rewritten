## Documentation regarding ENet communication

Enet is a standarized network protocol for game communication over UDP. It originate from Cube. I opted to use an Enet re-implementation in Rust for this project, with async support.

The generic Enet frame being decoded, it appears all the non-control data is composed of the "stable" type of data (with similar guarantee than TCP, but more optimised, especially for small packet, given that multiple one can be put in the same UDP packet)

The inner data is encrypted. It appears FlappetyFlap managed to decrypt them. On my side, I’ll be doing a bit of static reverse-engineering of the binary to figure out how to replace it.

### note about the binary

Ghidra erroneously marked a certain function as no return. Code after it is not decompiled, and I need to manually change the flow override for each call.
(can I make a script for it?)

- Blade.RN.Shared.NetData$$CloneProperty : appear to contain a list of types that message may contain.
- Blade.RN.Shared.ENetLib.Connection$$ComputeSecretAndActivateEncryption
- Blade.RN.Shared.ENetLib.Connection$$IsEncrypted
- Blade.RN.Shared.DiffieHellman$$CreateKeyPair
- Blade.RN.Shared.ENetLib.NetControllerThread$$WorkLoop
- Blade.RN.Shared.ENetLib.NetControllerThread$$ProcessOperations
- Blade.RN.Shared.ENetLib.SendPeerPacketOperation$$Execute
- Blade.RN.Shared.ENetLib.NetController$$SendPeerPacket
- Blade.RN.Client.ENetLib.NetMessageModule$$OnMessage
- Blade.RN.Shared.Crypto$$DecryptChaCha20InPlace

There is definitely a logging system set up.
