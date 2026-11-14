/**
 * DataSource - the common interface every layer must follow.
 *
 * Both the real file and all decorators implement this.
 * The client only talks to this — it never knows what's underneath.
 */
public interface DataSource {

    // Write data to whatever this source represents
    void writeData(String data);

    // Read data back from whatever this source represents
    String readData();
}
