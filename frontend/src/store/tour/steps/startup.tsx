import { StepName, StepsConfiguration } from '@/store/tour';

export const startupSteps: StepsConfiguration = {
    enumName: StepName.STARTUPSTEPS,
    list: [
        {
            selector: '.tour-step-0-startup',
            content:
                "Welcome 😄! We are glad you are here. Let's give you a tour on how things are done around here. The tutorial tours will guide you through our website 🤖. The tutorial can continue automatically, but when it requires you to click on the →, it will be mentioned. Please follow the steps in the tutorial for the best experience. Enjoy! (click →)",
        },
        {
            selector: '.tour-step-1-startup',
            content:
                'Firstly you can find some more information in the docs tab 📖. You can check that out later. (click →)',
        },
        {
            selector: '.tour-step-2-startup',
            content:
                'If you are not logged in yet you will see two buttons that say "Register" and "Login". Click on Register to continue. Otherwise you will see a Button that says "Dashboard". Once you click on Dashboard you will be redirected to the dashboard page and leave this tutorial. But not to worry, you will see a tutorial button where you can start some more tutorials. Bye👋!',
        },
        {
            selector: '.tour-step-3-startup',
            content: 'Fill in a username and password (and click →)',
        },
        {
            selector: '.tour-step-4-startup',
            content: 'Register',
        },
        {
            selector: '.tour-step-5-startup',
            content: 'Now you can log in',
        },
        {
            selector: '.tour-step-6-startup',
            content: 'Fill in your username and password (and click →)',
        },
        {
            selector: '.tour-step-7-startup',
            content:
                'Once you click on Login you will be redirected to the dashboard page and leave this tutorial. But not to worry, you will see a tutorial button where you can start some more tutorials. Bye👋!',
        },
    ],
};
