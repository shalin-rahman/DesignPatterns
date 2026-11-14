/**
 * Main - driver class that runs all four scenarios.
 *
 * Problem this solves:
 * We need to read and write files in different ways — sometimes plain,
 * sometimes encrypted, sometimes compressed, sometimes both. If we made
 * a separate class for every combination (EncryptedFile, CompressedFile,
 * EncryptedCompressedFile...) the code would explode. Instead, we use
 * the Decorator pattern: start with a plain FileDataSource and wrap it
 * with as many decorators as needed at runtime.
 *
 * Key point: the demo() method at the bottom never changes.
 * It calls writeData() and readData() on a DataSource and that is it.
 * Whether there are zero, one, or two decorators in the chain, the
 * client code looks exactly the same.
 *
 * Chain for Scenario 4:
 *   client -> CompressionDecorator -> EncryptionDecorator -> FileDataSource
 *
 *   write: compress -> encrypt -> save to file
 *   read:  load from file -> decrypt -> decompress
 */
public class Main {

    public static void main(String[] args) {
        final String data = "Hello, Decorator Pattern! This is some sample data for the file system demo.";

        // Scenario 1: no decorators — just a plain file
        System.out.println("=== Scenario 1: Plain file ===");
        DataSource plain = new FileDataSource("plain.txt");
        demo(plain, data);

        // Scenario 2: encrypted file — EncryptionDecorator wraps FileDataSource
        // write: Base64-encode -> save to file
        // read:  load from file -> Base64-decode
        System.out.println("=== Scenario 2: Encrypted file ===");
        DataSource encrypted = new EncryptionDecorator(
                new FileDataSource("encrypted.txt")
        );
        demo(encrypted, data);

        // Scenario 3: compressed file — CompressionDecorator wraps FileDataSource
        // write: GZIP compress -> save to file
        // read:  load from file -> GZIP decompress
        System.out.println("=== Scenario 3: Compressed file ===");
        DataSource compressed = new CompressionDecorator(
                new FileDataSource("compressed.txt")
        );
        demo(compressed, data);

        // Scenario 4: compressed AND encrypted — two decorators stacked
        // The outer decorator (Compression) runs first on write, last on read.
        // write: compress -> encrypt -> save to file
        // read:  load from file -> decrypt -> decompress
        System.out.println("=== Scenario 4: Compressed + Encrypted ===");
        DataSource both = new CompressionDecorator(
                new EncryptionDecorator(
                        new FileDataSource("compressed_encrypted.txt")
                )
        );
        demo(both, data);
    }

    /**
     * Shared client method. Writes data then reads it back.
     * Has no idea what decorators are in the chain — it just uses DataSource.
     */
    private static void demo(DataSource source, String data) {
        System.out.println("  WRITE:");
        source.writeData(data);

        System.out.println("  READ:");
        String result = source.readData();

        // If the round-trip is correct, read output matches the original input
        System.out.println("  Round-trip OK: " + data.equals(result));
        System.out.println();
    }
}
