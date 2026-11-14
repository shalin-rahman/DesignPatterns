import java.io.*;

/**
 * FileDataSource - the real file, the innermost layer.
 *
 * This is the actual thing being wrapped. It just reads and writes
 * raw bytes to disk. No encryption, no compression — nothing extra.
 *
 * Every decorator chain ends here. By the time data reaches this class,
 * the decorators above have already done their transformations.
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
