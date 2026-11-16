import java.io.File;

/**
 * EmailNotificationListener - Concrete Observer / Subscriber.
 *
 * Role in Observer Pattern:
 *   - Implements the EventListener interface.
 *   - Reacts to publisher updates by dispatching an email notification to the recipient.
 */
public class EmailNotificationListener implements EventListener {

    private final String email;

    /**
     * Constructs an email notification listener for a given email address.
     *
     * @param email destination email address.
     */
    public EmailNotificationListener(String email) {
        this.email = email;
    }

    /**
     * Handles the event by sending an email notification.
     *
     * @param eventType the event that occurred.
     * @param file      the file associated with the event.
     */
    @Override
    public void update(String eventType, File file) {
        System.out.println("Email to " + email + ": Someone has performed " + eventType
                + " operation with the following file: " + (file != null ? file.getName() : "unknown"));
    }

    public String getEmail() {
        return this.email;
    }
}
