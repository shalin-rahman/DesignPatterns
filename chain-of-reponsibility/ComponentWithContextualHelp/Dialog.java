/**
 * Dialog - Concrete Intermediate Handler.
 *
 * <p>Source: Refactoring Guru — Chain of Responsibility, GUI / Help-System example.
 *
 * <h2>Role in Chain of Responsibility Pattern</h2>
 * <ul>
 *   <li>A concrete container that can hold {@link Button}s (or other child components).</li>
 *   <li>When a child Button cannot handle a help request (no tooltip), the request
 *       arrives here via the chain.</li>
 *   <li>If the Dialog has a wiki page URL, it can open the relevant documentation — and
 *       the chain stops.  If not, the request propagates further up to the enclosing
 *       {@link Panel} or {@link Window}.</li>
 * </ul>
 *
 * <h2>Chain position</h2>
 * <pre>
 *   Button → [Dialog] → Panel → Window
 * </pre>
 *
 * <h2>Real-world analogy</h2>
 * The "Head Chef" in the restaurant chain: if the waiter (Button) couldn't resolve your
 * complaint, it reaches the Head Chef (Dialog).  If the Chef knows the answer, the
 * escalation stops here.
 */
public class Dialog extends Container {

    /**
     * URL to a wiki / documentation page that this dialog can open.
     * If set, this dialog is a terminating handler for help requests.
     */
    private final String wikiPageUrl;

    /**
     * Constructs a Dialog without a wiki page — help requests pass through to the parent.
     */
    public Dialog() {
        this.wikiPageUrl = null;
    }

    /**
     * Constructs a Dialog with an associated wiki page URL, making it a handler that
     * can terminate the chain.
     *
     * @param wikiPageUrl URL to the documentation page for this dialog's context.
     */
    public Dialog(String wikiPageUrl) {
        this.wikiPageUrl = wikiPageUrl;
    }

    /**
     * Handles the help request by opening the dialog's wiki page if available;
     * otherwise, forwards the request up to the parent container.
     *
     * <p>This override demonstrates the "handle-or-forward" decision at the container
     * level.  Compare to Observer: an Observer's {@code update()} always runs — it
     * never conditionally forwards.
     */
    @Override
    public void showHelp() {
        if (wikiPageUrl != null) {
            // This Dialog CAN handle the request — chain stops here.
            System.out.println("[Dialog] Opening wiki page for context-specific help: " + wikiPageUrl);
        } else {
            // No wiki page configured — pass up to the enclosing Panel / Window.
            System.out.println("[Dialog] No wiki page — forwarding to " +
                    (container != null ? container.getClass().getSimpleName() : "nobody (end of chain)") + "...");
            if (container != null) {
                container.showHelp();
            } else {
                System.out.println("[Dialog] End of chain — no help available.");
            }
        }
    }

    /**
     * Returns the wiki page URL, or {@code null} if unset.
     *
     * @return wiki page URL.
     */
    public String getWikiPageUrl() {
        return wikiPageUrl;
    }
}
