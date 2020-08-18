extern crate bitcoin_hashes;
extern crate hex;
extern crate secp256k1;

use bitcoin_hashes::{sha256, Hash};
use secp256k1::{Error, Message, PublicKey, Secp256k1, Signature, Verification, VerifyOnly};

/// VerifierError is the AuthVerifier errors.
pub enum VerifierError {
    Secp256k1Error(Error),
    HexError(hex::FromHexError),
}

/// AuthVerifier verifies the secp256k1 signature of a message with a given pubkey.
pub struct AuthVerifier {
    secp: Secp256k1<VerifyOnly>,
}

impl AuthVerifier {
    pub fn new() -> Self {
        AuthVerifier {
            secp: Secp256k1::verification_only(),
        }
    }

    /// verifies the secp256k1 signature of a message with a given pubkey.
    pub fn verify(self, hk1: &str, hsig: &str, hpubkey: &str) -> Result<bool, VerifierError> {
        let msg = hex::decode(hk1).map_err(|e| VerifierError::HexError(e))?;
        let sig = hex::decode(hsig).map_err(|e| VerifierError::HexError(e))?;
        let pubkey = hex::decode(hpubkey).map_err(|e| VerifierError::HexError(e))?;
        return verify_sig(&self.secp, &msg, &sig, &pubkey)
            .map_err(|e| VerifierError::Secp256k1Error(e));
    }
}

/// verify_sig checks if the signature of a key for a given message is valid.
pub fn verify_sig<C: Verification>(
    secp: &Secp256k1<C>,
    msg: &[u8],
    sig: &[u8],
    pubkey: &[u8],
) -> Result<bool, Error> {
    let msg = sha256::Hash::hash(msg);
    let msg = Message::from_slice(&msg)?;
    let sig = Signature::from_compact(sig)?;
    let pubkey = PublicKey::from_slice(pubkey)?;

    Ok(secp.verify(&msg, &sig, &pubkey).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate bitcoin_hashes;
    use secp256k1::{Error, Message, Secp256k1, SecretKey, Signature, Signing};

    fn sign<C: Signing>(
        secp: &Secp256k1<C>,
        msg: &[u8],
        seckey: [u8; 32],
    ) -> Result<Signature, Error> {
        let msg = sha256::Hash::hash(msg);
        let msg = Message::from_slice(&msg)?;
        let seckey = SecretKey::from_slice(&seckey)?;
        Ok(secp.sign(&msg, &seckey))
    }

    #[test]
    fn test_verify_sig() {
        let secp = Secp256k1::new();
        let seckey = [
            59, 148, 11, 85, 134, 130, 61, 253, 2, 174, 59, 70, 27, 180, 51, 107, 94, 203, 174,
            253, 102, 39, 170, 146, 46, 252, 4, 143, 236, 12, 136, 28,
        ];
        let pubkey = [
            2, 29, 21, 35, 7, 198, 183, 43, 14, 208, 65, 139, 14, 112, 205, 128, 231, 245, 41, 91,
            141, 134, 245, 114, 45, 63, 82, 19, 251, 210, 57, 79, 54,
        ];
        let msg = b"This is some message";

        let signature = sign(&secp, msg, seckey).unwrap();

        let serialize_sig = signature.serialize_compact();

        assert!(verify_sig(&secp, msg, &serialize_sig, &pubkey).unwrap());
    }
}
