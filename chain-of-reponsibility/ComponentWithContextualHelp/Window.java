/**
 * Window - Concrete Terminal Handler (top-level container / last resort).
 *
 * <p>Source: Refactoring Guru — Chain of Responsibility, GUI / Help-System example.
 *
 * <h2>Role in Chain of Responsibility Pattern</h2>
 * <ul>
 *   <li>The final link in the chain — if no lower-level component handled the help
 *       request, it arrives here.</li>
 *   <li>Always has a fallback help message (the application's global help / "Getting
 *       Started" guide), so the chain always terminates gracefully.</li>
 *   <li>Has no {@code container} (parent) — it is the root of the UI hierarchy.</li>
 * </ul>
 *
 * <h2>Chain position</h2>
 * <pre>
 *   Button -&gt; Dialog -&gt; Panel -&gt; [Window]  (end of chain)
 * </pre>
 *
 * <h2>Restaurant analogy</h2>
 * The Manager at the top of the escalation ladder.  When the Waiter and Head Chef both
 * couldn't resolve the complaint, it finally reaches the Manager who always has an answer
 * (even if it's just: "Here is our complaints policy booklet").
 *
 * <h2>Proxy vs. Window</h2>
 * A {@link Window} is not a Proxy.  A Proxy <em>impersonates</em> the real object and
 * decides whether to forward a call to it.  The Window <em>is</em> the real, final
 * handler — it doesn't stand in for anything else.
 */
public class Window extends Container {

    /**
     * The application-level / global help guide shown when no component-specific help exists.
     * This is the ultimate fallback — the Window always has something to show.
     */
    private final String globalHelpGuide;

    /**
     * Constructs the top-level application Window with a global help guide.
     *
     * @param globalHelpGuide text or URL for the application's general help documentation.
     */
    public Window(String globalHelpGuide) {
        this.globalHelpGuide = globalHelpGuide;
    }

    /**
     * Terminal handler: always shows the global help guide.
     *
     * <p>Unlike intermediate handlers which conditionally forward, the Window <em>always</em>
     * handles the request — it is the chain's guaranteed safety net.
     */
    @Override
    public void showHelp() {
        // The Window is the last resort — it always handles the request.
        System.out.println("[Window] No specific help found in the chain.");
        System.out.println("[Window] Displaying global help guide: \"" + globalHelpGuide + "\"");
    }

    /**
     * Returns the global help guide content.
     *
     * @return global help guide string.
     */
    public String getGlobalHelpGuide() {
        return globalHelpGuide;
    }
}
