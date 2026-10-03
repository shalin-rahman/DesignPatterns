package singletone;

public class MainClass {
    public static void main(String[] args) {
        // Problem: Two separate Admin objects are created for the same organization
        Admin_Problem admin1 = new Admin_Problem("Saeed", "saeed@iit.du.ac.bd");
        Admin_Problem admin2 = new Admin_Problem("Toukir", "toukir@gmail.com");

        System.out.println("Admin 1 Name: " + admin1.getName());
        System.out.println("Admin 2 Name: " + admin2.getName());

        // Check if they refer to the same object
        System.out.println("Are both admins the same instance? " + (admin1 == admin2)); 
        // Output: false (State is fragmented, policy updates could conflict)
    }


    // After Singletone introduce 
    // public static void main(String[] args) {
    //     Admin_Solve admin = Admin_Solve.getInstance();
    //     admin.setAdminDetails("Saeed Siddik", "saeed@gmail.com");
    //     admin.updateOrganizationPolicy("Remote Work Allowed");
    //     System.out.println("Admin Name: " + admin.getName()); // Outputs: Saeed

    //     // Any other class accessing getInstance() gets the exact same object
    //     Admin_Solve admin_2 = Admin_Solve.getInstance();
    //     admin_2.setAdminDetails("Toukir", "toukir@gmail.com");
    //     System.out.println("Admin Name: " + admin_2.getName()); 

    //     System.out.println("Admin Name: " + admin.getName()); 
    //     System.out.println("Same instance? " + (admin == admin_2)); // Outputs: true

    // }
}