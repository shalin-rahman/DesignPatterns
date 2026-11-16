import java.io.File;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * EventManager - Subject / Event Dispatcher (Publisher).
 *
 * Role in Observer Pattern:
 *   - Maintains a subscription registry (event type -> list of listeners).
 *   - Provides methods to dynamically subscribe and unsubscribe listeners.
 *   - Iterates through registered listeners and broadcasts notifications when events occur.
 */
public class EventManager {

    /**
     * Map associating each event name with its list of subscribed listeners.
     */
    private final Map<String, List<EventListener>> listeners = new HashMap<>();

    /**
     * Initializes the EventManager with supported event operations.
     *
     * @param operations array of event names supported (e.g., "open", "save").
     */
    public EventManager(String... operations) {
        for (String operation : operations) {
            this.listeners.put(operation, new ArrayList<>());
        }
    }

    /**
     * Subscribes a listener to a specific event type.
     *
     * @param eventType the event to subscribe to.
     * @param listener  the subscriber implementing EventListener.
     */
    public void subscribe(String eventType, EventListener listener) {
        List<EventListener> users = listeners.get(eventType);
        if (users != null) {
            users.add(listener);
        } else {
            System.err.println("[EventManager] Unknown event type: " + eventType);
        }
    }

    /**
     * Unsubscribes a listener from a specific event type.
     *
     * @param eventType the event to unsubscribe from.
     * @param listener  the subscriber to remove.
     */
    public void unsubscribe(String eventType, EventListener listener) {
        List<EventListener> users = listeners.get(eventType);
        if (users != null) {
            users.remove(listener);
        }
    }

    /**
     * Broadcasts an event notification to all listeners subscribed to the event type.
     *
     * @param eventType the event type that occurred.
     * @param file      the file associated with the event.
     */
    public void notify(String eventType, File file) {
        List<EventListener> users = listeners.get(eventType);
        if (users != null) {
            for (EventListener listener : users) {
                listener.update(eventType, file);
            }
        }
    }
}
