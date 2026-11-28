import java.util.ArrayList;
import java.util.List;

/**
 * Composite handler that holds children and auto-wires their successor links upon addition.
 */
public abstract class Container extends Component {

    private final List<Component> children = new ArrayList<>();

    /**
     * Adds a child component and auto-wires this container as the child's next handler.
     */
    public void add(Component child) {
        children.add(child);
        child.setContainer(this);
    }

    public List<Component> getChildren() {
        return children;
    }
}
