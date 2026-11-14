import java.io.*;
import java.util.Base64;
import java.util.zip.*;

/**
 * CompressionDecorator - Concrete Decorator (Concrete Wrapper) for GZIP compression.
 *
 * Pattern Terminology:
 *   - WRAPPER / DECORATOR: CompressionDecorator wraps and decorates any DataSource (the wrappee)
 *              to provide transparent data compression and decompression.
 *   - WRAPPEE / DECORATED COMPONENT: The DataSource instance provided via constructor (source), which can
 *              be a raw FileDataSource or another wrapper/decorator (e.g. EncryptionDecorator).
 *   - WRAP / DECORATE: Dynamically decorates/wraps the data stream by compressing on write
 *              and decompressing on read.
 *
 * Interception Flow:
 *   - writeData (Wrapper/Decorator -> Wrappee/Decorated Component):
 *       1. Wrapper/Decorator compresses the string into GZIP bytes and Base64 encodes them.
 *       2. Wrapper/Decorator passes the compressed data inward to the wrappee (super.writeData).
 *   - readData (Wrappee/Decorated Component -> Wrapper/Decorator):
 *       1. Wrapper/Decorator fetches data from the wrappee (super.readData).
 *       2. Wrapper/Decorator decodes Base64 and decompresses GZIP bytes back to original text.
 */
public class CompressionDecorator extends DataSourceDecorator {

    /**
     * Initializes the compression wrapper/decorator around the target wrappee/component.
     *
     * @param source The wrappee being decorated and wrapped (FileDataSource or another wrapper/decorator)
     */
    public CompressionDecorator(DataSource source) {
        super(source); // stores wrappee in base class
    }

    @Override
    public void writeData(String data) {
        // Step 1: Wrapper / Decorator performs compression & Base64 encoding
        byte[] compressed = compress(data);
        String encoded = Base64.getEncoder().encodeToString(compressed);
        System.out.printf("  [CompressionDecorator] Compressed %d chars -> %d chars%n",
                data.length(), encoded.length());

        // Step 2: Wrapper / Decorator forwards compressed data to the wrappee / decorated component
        super.writeData(encoded);
    }

    @Override
    public String readData() {
        // Step 1: Wrapper / Decorator retrieves compressed data from its wrappee / decorated component
        String encoded = super.readData();

        // Step 2: Wrapper / Decorator decodes Base64 and decompresses the GZIP payload
        byte[] compressed = Base64.getDecoder().decode(encoded);
        String original = decompress(compressed);
        System.out.printf("  [CompressionDecorator] Decompressed %d chars -> %d chars%n",
                encoded.length(), original.length());
        return original;
    }

    // Compresses a string using GZIP and returns the raw byte array
    private byte[] compress(String data) {
        try (ByteArrayOutputStream bos = new ByteArrayOutputStream();
             GZIPOutputStream gzip = new GZIPOutputStream(bos)) {
            gzip.write(data.getBytes());
            gzip.finish();
            return bos.toByteArray();
        } catch (IOException e) {
            throw new RuntimeException("Compression failed: " + e.getMessage(), e);
        }
    }

    // Decompresses a GZIP byte array and returns the original string
    private String decompress(byte[] compressed) {
        try (ByteArrayInputStream bis = new ByteArrayInputStream(compressed);
             GZIPInputStream gzip = new GZIPInputStream(bis);
             ByteArrayOutputStream bos = new ByteArrayOutputStream()) {
            byte[] buffer = new byte[1024];
            int len;
            while ((len = gzip.read(buffer)) != -1) {
                bos.write(buffer, 0, len);
            }
            return bos.toString();
        } catch (IOException e) {
            throw new RuntimeException("Decompression failed: " + e.getMessage(), e);
        }
    }
}
