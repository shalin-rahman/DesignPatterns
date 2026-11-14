import java.util.Base64;

/**
 * EncryptionDecorator - adds encryption on top of any DataSource.
 *
 * Write: encodes the data to Base64, then passes it to the next layer.
 * Read:  gets the encoded data from the next layer, decodes it back.
 *
 * We use Base64 to keep the example simple and dependency-free.
 * In a real system this would be AES or RSA encryption.
 *
 * The pattern for every content-modifying decorator is always:
 *   writeData: transform first, then delegate inward
 *   readData:  delegate inward first, then reverse-transform on the way out
 */
public class EncryptionDecorator extends DataSourceDecorator {

    public EncryptionDecorator(DataSource source) {
        super(source);
    }

    @Override
    public void writeData(String data) {
        // Encode the plain data to Base64
        String encoded = Base64.getEncoder().encodeToString(data.getBytes());
        System.out.printf("  [EncryptionDecorator] Encrypted: '%s' -> '%s'%n", data, encoded);

        // Pass the encoded data down to the next layer (file or another decorator)
        super.writeData(encoded);
    }

    @Override
    public String readData() {
        // Get whatever the layer below stored (encoded data)
        String encoded = super.readData();

        // Decode it back to the original plain text
        String decoded = new String(Base64.getDecoder().decode(encoded));
        System.out.printf("  [EncryptionDecorator] Decrypted: '%s' -> '%s'%n", encoded, decoded);
        return decoded;
    }
}
