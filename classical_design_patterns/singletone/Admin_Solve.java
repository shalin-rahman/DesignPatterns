package singletone;

import java.util.ArrayList;
import java.util.List;

public class Admin_Solve {
    private String name;
    private String email;
    private final List<String> policies;

    // 1. Private constructor prevents instantiation from outside
    private Admin_Solve() {
        this.name = "Default Admin";
        this.email = "admin@org.com";
        this.policies = new ArrayList<>();
    }

    // 2. Static inner class holds the Singleton instance (Lazy & Thread-safe)
    private static class SingletonHolder {
        private static final Admin_Solve INSTANCE = new Admin_Solve();
    }

    // 3. Global access point to get the single instance
    public static Admin_Solve getInstance() {
        return SingletonHolder.INSTANCE;
    }

    // Setter to update details for the single instance
    public void setAdminDetails(String name, String email) {
        this.name = name;
        this.email = email;
    }

    public void updateOrganizationPolicy(String policy) {
        policies.add(policy);
        System.out.println("Admin " + name + " updated policy to: " + policy);
    }

    public String getName() {
        return name;
    }

    public String getEmail() {
        return email;
    }

    public List<String> getPolicies() {
        return policies;
    }
}