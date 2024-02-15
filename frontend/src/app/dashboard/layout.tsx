import DashboardNavbar from "@/components/DashboardNavbar";
import DashboardSidebar from "@/components/DashboardSidebar";

export default function DashboardLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <>
            <DashboardNavbar />
            <div className="flex h-full">
                <DashboardSidebar />
                <div className="grow px-12 py-8">
                    {children}
                </div>
            </div>
        </>
    );
}
