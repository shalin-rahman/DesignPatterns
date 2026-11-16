import java.io.File;

/**
 * LogOpenListener - Concrete Observer / Subscriber.
 *
 * Role in Observer Pattern:
 *   - Implements the EventListener interface.
 *   - Reacts to publisher updates by logging the operation details to a log file.
 */
public class LogOpenListener implements EventListener {

    private final File log;

    /**
     * Constructs a log listener writing to a specified file path.
     *
     * @param fileName target log file path or name.
     */
    public LogOpenListener(String fileName) {
        this.log = new File(fileName);
    }

    /**
     * Handles the event by writing or outputting the log record.
     *
     * @param eventType the event that occurred.
     * @param file      the file associated with the event.
     */
    @Override
    public void update(String eventType, File file) {
        System.out.println("Save to log " + log + ": Someone has performed " + eventType
                + " operation with the following file: " + (file != null ? file.getName() : "unknown"));
    }

    public File getLog() {
        return this.log;
    }
}
