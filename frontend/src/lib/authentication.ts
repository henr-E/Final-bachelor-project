import {
    createChannel,
    createClient,
    ClientError,
    Status,
} from "nice-grpc-web";
import { Result, Err, Ok } from "ts-results";
import { z } from "zod";

import {
    AuthenticationServiceDefinition,
    AuthenticationServiceClient,
    LoginError as GrpcLoginError,
    RegisterError as GrpcRegisterError,
    User
} from "@/proto/authentication/auth";
import {BackendLogin, BackendRegister} from "@/api/user/crud";

// Validate login and register form data with the same schema as they use the
// same data. Define more schemas if necessary.
const schema = z.object({
    username: z.string({
        required_error: "username is required",
    }),
    password: z.string(),
});

export interface ZodFormValidationLoginErrors {
    username?: string[];
    password?: string[];
}

export interface ZodFormValidationRegisterErrors {
    username?: string[];
    password?: string[];
    confirmPassword?: string[];
}

// all types of error that can occur

interface LoginErrorForm {
    message: "form";
    error: ZodFormValidationLoginErrors;
}
interface LoginErrorGrpc {
    message: "Invalid credentials";
    error: GrpcLoginError;
}
export type LoginError = LoginErrorForm | LoginErrorGrpc;

interface RegisterErrorForm {
    message: "form";
    error: ZodFormValidationRegisterErrors;
}
interface RegisterErrorGrpc {
    message: "Username is taken";
    error: GrpcRegisterError;
}
export type RegisterError = RegisterErrorForm | RegisterErrorGrpc;

export async function login(
    formData: FormData
): Promise<Result<string, LoginError>> {
    // Create and validate the user data input schema from form data.


    const user = schema.safeParse({
        username: formData.get("username"),
        password: formData.get("password"),
    });


    // Return errors belonging to schema validation if any.
    if (!user.success) {
        return Err({ message: "form", error: user.error.flatten().fieldErrors });
    }


    const loginResponse = await BackendLogin(user.data);

    // Return expected errors if they were returned above, else map the
    // loginResponse type to the token field on the type.
    if (loginResponse.error !== undefined) {
        // Return the error to the caller
        return Err({ message: "Invalid credentials", error: loginResponse.error });
    } else if (loginResponse.token) {
        // Return the json webtoken
        return Ok(loginResponse.token);
    } else {
        throw Error("unreachable");
    }
}

export async function register(
    formData: FormData): Promise<Result<{}, RegisterError>> {

    // Create and validate the user data input schema from form data.
    const user = schema.safeParse({
        username: formData.get("username"),
        password: formData.get("password"),
    });
    // Return errors belonging to schema validation if any.
    if (!user.success) {
        return Err({ message: "form", error: user.error.flatten().fieldErrors });
    }

    const registerResponse = await BackendRegister(user.data);

    // Return expected errors if they were returned above, else map the
    // loginResponse type to the token field on the type.
    if (registerResponse.error !== undefined) {
        // Return the error to the caller
        return Err({ message: "Username is taken", error: registerResponse.error });
    } else {
        return Ok(registerResponse)
    }
}
