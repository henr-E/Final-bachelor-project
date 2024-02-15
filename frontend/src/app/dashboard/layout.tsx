import DashboardNavbar from "@/components/DashboardNavbar";
import DashboardSidebar from "@/components/DashboardSidebar";

export default function DashboardLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <div className="flex flex-col h-screen">
            <DashboardNavbar />
            <div className="flex grow">
                <DashboardSidebar />
                <div className="grow px-12 py-8">
                    {children}
                </div>
            </div>
        </div>
    );
}
