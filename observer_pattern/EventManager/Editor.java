import java.io.File;

/**
 * Editor - Concrete Publisher / Context.
 *
 * Role in Observer Pattern:
 *   - Represents the business logic component whose state changes trigger events.
 *   - Contains a reference to an EventManager instance, allowing clients to configure subscriptions.
 *   - Notifies subscribers via its EventManager whenever open or save operations occur.
 */
public class Editor {

    /**
     * Public event manager allowing external clients to subscribe/unsubscribe listeners.
     */
    public final EventManager events;

    /**
     * Currently active file in the editor.
     */
    private File file;

    /**
     * Initializes the editor and configures supported event types ("open", "save").
     */
    public Editor() {
        this.events = new EventManager("open", "save");
    }

    /**
     * Opens a file and notifies all "open" event listeners.
     *
     * @param filePath path to the file being opened.
     */
    public void openFile(String filePath) {
        this.file = new File(filePath);
        events.notify("open", this.file);
    }

    /**
     * Saves the current file and notifies all "save" event listeners.
     *
     * @throws Exception if save is attempted without an open file.
     */
    public void saveFile() throws Exception {
        if (this.file != null) {
            events.notify("save", this.file);
        } else {
            throw new Exception("Please open a file first.");
        }
    }

    /**
     * Retrieves the currently open file.
     *
     * @return the active File object, or null if no file is open.
     */
    public File getFile() {
        return this.file;
    }
}
