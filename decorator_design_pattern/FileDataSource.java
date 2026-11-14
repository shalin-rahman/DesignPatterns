import java.io.*;

/**
 * FileDataSource - the Concrete Component (innermost Wrappee / Decorated Core).
 *
 * Pattern Terminology:
 *   - WRAPPEE / DECORATED COMPONENT: FileDataSource is the foundational target at the core of the chain.
 *              It is the object that gets wrapped and decorated. Unlike wrappers/decorators, it is purely
 *              a wrappee and never wraps or decorates anything else (holds no inner DataSource reference).
 *   - WRAP / DECORATE: While wrappers decorate this component with added features (encryption, compression),
 *              FileDataSource itself performs raw, undecorated disk operations.
 *
 * This class performs the actual I/O: reading and writing raw bytes directly to disk.
 * Every decorator/wrapper chain eventually delegates here once all wrapper transformations
 * (e.g. encryption, compression) are complete.
 */
public class FileDataSource implements DataSource {

    private final String filename;

    public FileDataSource(String filename) {
        this.filename = filename;
    }

    /**
     * Writes data straight to disk. No changes at all.
     * Whatever arrives here is exactly what gets saved to the file.
     */
    @Override
    public void writeData(String data) {
        File file = new File(filename);
        try (OutputStream fos = new FileOutputStream(file)) {
            fos.write(data.getBytes(), 0, data.length());
            System.out.printf("  [FileDataSource] Wrote %d bytes to '%s'%n", data.length(), filename);
        } catch (IOException ex) {
            System.out.println("  [FileDataSource] Write failed: " + ex.getMessage());
        }
    }

    /**
     * Reads all bytes from disk and returns them as a string.
     * No changes — raw data comes back exactly as it was stored.
     * Decorators above will reverse-transform it on the way out.
     */
    @Override
    public String readData() {
        char[] buffer;
        File file = new File(filename);
        try (FileReader reader = new FileReader(file)) {
            buffer = new char[(int) file.length()];
            reader.read(buffer);
        } catch (IOException ex) {
            System.out.println("  [FileDataSource] Read failed: " + ex.getMessage());
            return "";
        }
        System.out.printf("  [FileDataSource] Read %d bytes from '%s'%n", file.length(), filename);
        return new String(buffer);
    }
}
