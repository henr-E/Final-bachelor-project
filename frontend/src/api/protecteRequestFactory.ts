import {
    ClientError,
    ClientMiddlewareCall,
    createClientFactory,
    Metadata,
    Status,
} from 'nice-grpc-web';
import ToastNotification from '@/components/notification/ToastNotification';
import { CallOptions } from 'nice-grpc-common';

/**
 * Checks if request sends an UNAUTHENTICATED error back, when it does redirect the user to the login page and delete his current token
 * @param call
 * @param options
 */
async function* loginStatusCheckerMiddleware<Request, Response>(
    call: ClientMiddlewareCall<Request, Response>,
    options: CallOptions
) {
    const { path } = call.method;

    try {
        return yield* call.next(call.request, options);
    } catch (error) {
        if (
            error instanceof ClientError &&
            Status[error.code] == 'UNAUTHENTICATED' &&
            document.location.pathname != '/' &&
            document.location.pathname != '/docs'
        ) {
            localStorage.removeItem('authToken');
            ToastNotification('error', 'Not singed in');
            document.location.href = '/';
        }
        throw error;
    }
}

/**
 * Factory that adds the user token to the request and redirects when user is not authenticated
 */
export const clientAuthLayer = createClientFactory()
    .use((call, options) => {
        return call.next(call.request, {
            ...options,
            metadata: Metadata(options.metadata).set(
                'Authorization-token',
                `${localStorage.getItem('authToken')}`
            ),
        });
    })
    .use(loginStatusCheckerMiddleware);
