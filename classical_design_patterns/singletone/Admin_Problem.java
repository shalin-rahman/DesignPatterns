package singletone;

class Admin_Problem {
    private String name;
    private String email;

    // Public constructor allows unlimited instances
    public Admin_Problem(String name, String email) {
        this.name = name;
        this.email = email;
    }

    public void updateOrganizationPolicy(String policy) {
        System.out.println("Admin " + name + " updated policy to: " + policy);
    }

    public String getName() {
        return name;
    }
}
