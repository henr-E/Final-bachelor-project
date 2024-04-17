/** @type {import('next').NextConfig} */
const nextConfig = {
    reactStrictMode: false,
    redirects: () => [
        {
            source: '/dashboard',
            destination: '/dashboard/overview',
            permanent: false,
        },
    ],
    output: 'standalone',
};

export default nextConfig;
