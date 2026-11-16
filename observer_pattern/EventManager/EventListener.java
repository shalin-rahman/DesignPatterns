import java.io.File;

/**
 * EventListener - Observer / Subscriber Interface.
 *
 * Role in Observer Pattern:
 *   - Declares the notification interface that all concrete subscribers must implement.
 *   - Allows the EventManager (Publisher) to broadcast events to any listener without
 *     knowing its concrete class (loose coupling).
 */
public interface EventListener {

    /**
     * Called by the event publisher when a subscribed event occurs.
     *
     * @param eventType the type/name of the triggered event (e.g., "open", "save").
     * @param file      the file associated with the event.
     */
    void update(String eventType, File file);
}
