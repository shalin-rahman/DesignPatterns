/**
 * DataSource - the common Component interface every layer must follow.
 *
 * Both the Wrapper (Decorator) and the Wrappee (Decorated Component / Target)
 * implement this interface. Because both share the same contract:
 *   - The client treats any Wrapper (Decorator) or Wrappee (Decorated Object) identically.
 *   - A Wrapper (Decorator) can wrap/decorate another Wrapper/Decorator just as easily as a raw file.
 *
 * Roles:
 *   - Component Interface: Defines writeData() and readData().
 *   - Unifies Wrappers (Decorators) and Wrappees (Decorated Components / inner Decorators).
 */
public interface DataSource {

    // Write data to whatever this source represents
    void writeData(String data);

    // Read data back from whatever this source represents
    String readData();
}
