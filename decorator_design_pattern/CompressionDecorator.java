import java.io.*;
import java.util.Base64;
import java.util.zip.*;

/**
 * CompressionDecorator - adds GZIP compression on top of any DataSource.
 *
 * Write: compresses the data, Base64-encodes the result, passes it down.
 * Read:  gets data from below, Base64-decodes it, then decompresses.
 *
 * Why Base64 after GZIP?
 * GZIP produces raw binary bytes that cannot be safely stored in a Java String.
 * Base64 converts those bytes into printable characters so the compressed
 * result can travel through the chain and into the file cleanly.
 *
 * Order matters when stacking decorators.
 * If the chain is: CompressionDecorator -> EncryptionDecorator -> FileDataSource
 *   write: compress first, then encrypt, then write to file
 *   read:  read from file, decrypt, then decompress
 * Each decorator undoes its own step in reverse on the way back out.
 */
public class CompressionDecorator extends DataSourceDecorator {

    public CompressionDecorator(DataSource source) {
        super(source);
    }

    @Override
    public void writeData(String data) {
        // Step 1: GZIP compress
        byte[] compressed = compress(data);

        // Step 2: Base64-encode so the compressed bytes are safe to pass as a String
        String encoded = Base64.getEncoder().encodeToString(compressed);
        System.out.printf("  [CompressionDecorator] Compressed %d chars -> %d chars%n",
                data.length(), encoded.length());

        // Step 3: Hand the compressed+encoded string to the next layer
        super.writeData(encoded);
    }

    @Override
    public String readData() {
        // Step 1: Get the compressed+encoded data from the layer below
        String encoded = super.readData();

        // Step 2: Base64-decode back to raw GZIP bytes
        byte[] compressed = Base64.getDecoder().decode(encoded);

        // Step 3: GZIP decompress to restore the original string
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
