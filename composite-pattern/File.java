import java.util.Collections;
import java.util.List;
import java.util.Locale;
import java.util.Objects;

/** Leaf in the composite: a file has a fixed size and searchable content. */
public class File implements IFileSystemItem {
    private final String name;
    private final int size;
    private final String content;

    public File(String name, int size, String content) {
        this.name = Objects.requireNonNull(name, "name");
        this.content = Objects.requireNonNull(content, "content");
        if (size < 0) {
            throw new IllegalArgumentException("File size cannot be negative.");
        }
        this.size = size;
    }

    @Override
    public String getName() {
        return name;
    }

    @Override
    public int getSize() {
        return size;
    }

    @Override
    public void display(int indentationLevel) {
        System.out.println(indentation(indentationLevel) + name + " (" + size + " KB)");
    }

    @Override
    public List<File> search(String keyword) {
        String normalizedKeyword = Objects.requireNonNull(keyword, "keyword")
                .toLowerCase(Locale.ROOT);
        if (content.toLowerCase(Locale.ROOT).contains(normalizedKeyword)) {
            return Collections.singletonList(this);
        }
        return Collections.emptyList();
    }

    private static String indentation(int level) {
        StringBuilder indentation = new StringBuilder();
        for (int i = 0; i < level; i++) {
            indentation.append("  ");
        }
        return indentation.toString();
    }
}
