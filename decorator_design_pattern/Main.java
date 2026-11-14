/**
 * Main - driver class demonstrating Decorator (Wrapper/Wrappee) composition.
 *
 * Core Terminology:
 *   - WRAPPER / DECORATOR: An object that wraps and decorates a component or another
 *     decorator to attach dynamic responsibilities (e.g. EncryptionDecorator, CompressionDecorator).
 *   - WRAPPEE / DECORATED COMPONENT: The target component enclosed and decorated inside
 *     a wrapper (e.g. FileDataSource, or an inner decorator).
 *   - WRAP / DECORATE: The dynamic composition process: wrapping/decorating an object at runtime
 *     rather than static compile-time inheritance.
 *
 * Problem this solves:
 * Subclassing every combination (EncryptedFile, CompressedFile, EncryptedCompressedFile)
 * causes class explosion. The Decorator pattern instead wraps and decorates an innermost
 * Wrappee with one or more Wrappers/Decorators at runtime.
 *
 * Key insight (Chain Duality):
 * In a nested chain, an intermediate object is BOTH a Wrapper/Decorator and a Wrappee/Decorated component:
 *   - CompressionDecorator is a WRAPPER/DECORATOR wrapping and decorating EncryptionDecorator (its wrappee).
 *   - EncryptionDecorator is a WRAPPER/DECORATOR wrapping and decorating FileDataSource (its wrappee),
 *     while also acting as the WRAPPEE/DECORATED target to CompressionDecorator.
 *
 * Client Transparency:
 * The client code (demo method) interacts solely through the DataSource interface,
 * completely unaware of whether it is speaking to a Wrapper/Decorator or a raw Wrappee.
 */
public class Main {

    public static void main(String[] args) {
        final String data = "Hello, Decorator Pattern! This is some sample data for the file system demo.";

        // Scenario 1: Innermost Wrappee only (no wrappers / undecorated)
        // FileDataSource is the raw base component without any decoration/wrapping.
        System.out.println("=== Scenario 1: Plain file ===");
        DataSource plain = new FileDataSource("plain.txt");
        demo(plain, data);

        // Scenario 2: Single Wrapper/Decorator wrapping/decorating a Wrappee
        // Wrapper / Decorator: EncryptionDecorator
        // Wrappee / Decorated Component: FileDataSource
        System.out.println("=== Scenario 2: Encrypted file ===");
        DataSource encrypted = new EncryptionDecorator(
                new FileDataSource("encrypted.txt") // wrappee passed into wrapper to decorate it
        );
        demo(encrypted, data);

        // Scenario 3: Single Wrapper/Decorator wrapping/decorating a Wrappee
        // Wrapper / Decorator: CompressionDecorator
        // Wrappee / Decorated Component: FileDataSource
        System.out.println("=== Scenario 3: Compressed file ===");
        DataSource compressed = new CompressionDecorator(
                new FileDataSource("compressed.txt") // wrappee passed into wrapper to decorate it
        );
        demo(compressed, data);

        // Scenario 4: Stacked Wrappers/Decorators (Nested Wrapper -> Wrappee chain)
        // Outer Wrapper / Decorator: CompressionDecorator
        // Middle Layer:  EncryptionDecorator (Wrappee/Decorated by Compression; Wrapper/Decorator to FileDataSource)
        // Innermost:     FileDataSource (Innermost Wrappee / Decorated Core Component)
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
