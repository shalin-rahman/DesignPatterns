import java.util.Base64;

/**
 * EncryptionDecorator - Concrete Decorator (Concrete Wrapper) for encryption.
 *
 * Pattern Terminology:
 *   - WRAPPER / DECORATOR: EncryptionDecorator wraps and decorates any DataSource (the wrappee)
 *              to add Base64 encoding/decryption transparently.
 *   - WRAPPEE / DECORATED COMPONENT: The DataSource passed into the constructor (source) and stored
 *              in super.wrappee. It can be FileDataSource or another wrapper/decorator.
 *   - WRAP / DECORATE: Dynamically decorates/wraps data streams by encrypting during write
 *              and decrypting during read.
 *
 * Interception Flow:
 *   - writeData (Wrapper/Decorator -> Wrappee/Decorated Component):
 *       1. Wrapper/Decorator encrypts/encodes data.
 *       2. Wrapper/Decorator delegates the transformed data down to the wrappee (super.writeData).
 *   - readData (Wrappee/Decorated Component -> Wrapper/Decorator):
 *       1. Wrapper/Decorator requests raw data from the wrappee (super.readData).
 *       2. Wrapper/Decorator decrypts/decodes the data before returning it up the chain.
 */
public class EncryptionDecorator extends DataSourceDecorator {

    /**
     * Initializes the encryption wrapper/decorator around the target wrappee/component.
     *
     * @param source The wrappee being decorated and wrapped (FileDataSource or another wrapper/decorator)
     */
    public EncryptionDecorator(DataSource source) {
        super(source); // passes wrappee to base DataSourceDecorator
    }

    @Override
    public void writeData(String data) {
        // Step 1: Wrapper / Decorator performs pre-processing (Base64 encoding)
        String encoded = Base64.getEncoder().encodeToString(data.getBytes());
        System.out.printf("  [EncryptionDecorator] Encrypted: '%s' -> '%s'%n", data, encoded);

        // Step 2: Wrapper / Decorator delegates processed data inward to its wrappee / decorated component
        super.writeData(encoded);
    }

    @Override
    public String readData() {
        // Step 1: Wrapper / Decorator delegates inward to retrieve data from its wrappee / decorated component
        String encoded = super.readData();

        // Step 2: Wrapper / Decorator performs post-processing (Base64 decoding)
        String decoded = new String(Base64.getDecoder().decode(encoded));
        System.out.printf("  [EncryptionDecorator] Decrypted: '%s' -> '%s'%n", encoded, decoded);
        return decoded;
    }
}
