/**
 * Concrete terminal handler at the root of the hierarchy; acts as the guaranteed fallback.
 */
public class Window extends Container {

    private final String globalHelpGuide;

    public Window(String globalHelpGuide) {
        this.globalHelpGuide = globalHelpGuide;
    }

    /**
     * Always displays the application-wide help guide, terminating the chain.
     */
    @Override
    public void showHelp() {
        System.out.println("[Window] No specific help found in the chain.");
        System.out.println("[Window] Displaying global help guide: \"" + globalHelpGuide + "\"");
    }

    public String getGlobalHelpGuide() {
        return globalHelpGuide;
    }
}
