import React from 'react';
import Image from 'next/image';
import { List, ListItem } from 'flowbite-react';

interface SectionRefs {
    [key: string]: React.RefObject<HTMLDivElement>;
}

interface ContentSectionProps {
    refs: SectionRefs;
}

const ContentSection: React.FC<ContentSectionProps> = ({ refs }) => {
    return (
        <div className='flex flex-col items-center justify-center'>
            <div className={'w-full max-w-4xl space-y-10'}>
                {/*createAccount*/}
                <h1
                    ref={refs.createAccount}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Creating a user and logging in.
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>1. Click on Register</h3>
                    <p>
                        If you are not logged in yet, you will see two buttons that say
                        &quot;Register&quot; and &quot;Login&quot;. Click on Register to continue.
                        Otherwise you will see a Button that says &quot;Dashboard&quot;. Once you
                        click on Dashboard, you will be redirected to the dashboard page and leave
                        this tutorial.
                    </p>
                    <Image
                        alt='1'
                        src='/documentation/create_account/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        2. Follow the instructions in the modal and click Register
                    </h3>
                    <Image
                        alt='2'
                        src='/documentation/create_account/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>3. Click on Login</h3>
                    <Image
                        alt='3'
                        src='/documentation/create_account/3.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        4. Follow the instructions in the modal and click Login.
                    </h3>
                    <Image
                        alt='4'
                        src='/documentation/create_account/4.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>5. Click the logout button to logout</h3>
                    <Image
                        alt='4'
                        src='/documentation/create_account/5.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                {/*createTwin*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.createTwin}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Creating a twin.
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>1. Click on Select Twin</h3>
                    <Image
                        alt='1'
                        src='/documentation/create_twin/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>2. Click on Create Twin</h3>
                    <Image
                        alt='2'
                        src='/documentation/create_twin/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        3. Enter either the name of the city/place or its coordinates, then proceed
                        to click Search. You are not required to select coordinates if you input a
                        place name, and vice versa.
                    </h3>
                    <Image
                        alt='3'
                        src='/documentation/create_twin/3.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        4. Change the radius and fill in a custom name if you want. You can choose a
                        radius between 400 meters and 1000 meters
                    </h3>
                    <Image
                        alt='4'
                        src='/documentation/create_twin/4.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        5. Check whether the settings of the twin are correct and click on Create
                    </h3>
                    <Image
                        alt='5'
                        src='/documentation/create_twin/5.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                {/*switch_twin*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.switchTwin}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Switch to another twin
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>
                        Click on Select Twin or on the current twin. A dropdown will open where you
                        can click the twin you would like to switch to.
                    </h3>
                    <Image
                        alt='1'
                        src='/documentation/switch_twin/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                {/*deleteTwin*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.deleteTwin}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Deleting twins.
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>
                        1. To delete twins, use the checkboxes to select which ones you would like
                        to delete and click Delete selected twins.
                    </h3>
                    <Image
                        alt='1'
                        src='/documentation/delete_twin/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>2. Click on Delete.</h3>
                    <Image
                        alt='2'
                        src='/documentation/delete_twin/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                {/*realtime*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.realtime}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Realtime page
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>
                        1. Use the dropdown in the left down corner to choose a signal.
                    </h3>
                    <Image
                        alt='1'
                        src='/documentation/realtime/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>2. Choose a signal from the list</h3>
                    <p>
                        Every sensor is composed of a list of signals. A signal consists of the
                        quantity being measured, the unit it is measured in and the prefix
                        (factor/fraction) of the value that is ingested. Here you can select any
                        quantity supported by the system. If sensors measuring that quantity are
                        found, a live value heatmap will be displayed of those sensors.
                    </p>
                    <Image
                        alt='2'
                        src='/documentation/realtime/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                {/*createPreset*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.createPreset}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Create presets
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>1. Click on the + icon</h3>
                    <Image
                        alt='1'
                        src='/documentation/create_preset/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        2. Select the items for the preset using the checkboxes.
                    </h3>
                    <p>
                        Simulators work with graphs of nodes and edges. Both edges and nodes can
                        have components attached to them. Here you can select a set of components
                        that will be configured for you when you apply a preset to a building. You
                        can&apos;t add a node and an edge to the same preset. This will give a
                        warning.
                    </p>
                    <Image
                        alt='2'
                        src='/documentation/create_preset/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>3. Set the preset name and variables.</h3>
                    <Image
                        alt='3'
                        src='/documentation/create_preset/3.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                {/*deleteBuilding*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.deleteBuilding}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Delete a building.
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>1. Click on Delete building</h3>
                    <Image
                        alt='1'
                        src='/documentation/delete_building/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        2. Click on Restore building to undo the deletion
                    </h3>
                    <Image
                        alt='2'
                        src='/documentation/delete_building/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                {/*createSensor*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.createBuildingSensor}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Create a building sensor from the editor tab.
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>1. Click on Create Sensor</h3>
                    <p>A sensor tries to emulate a household in our digital twin.</p>
                    <Image
                        alt='1'
                        src='/documentation/create_sensor/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        2. Fill in the name, description and click Next.
                    </h3>
                    <Image
                        alt='2'
                        src='/documentation/create_sensor/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        3. Add the desired signals. You can delete added signals by pressing the X
                        on the corresponding signal. Click next
                    </h3>
                    <p>
                        Every sensor is required to have a{' '}
                        <text className='font-bold underline'>TIMESTAMP</text>
                        signal. This is enforced by the frontend.
                    </p>
                    <br />
                    <p>
                        If a (sensor)load node is placed on a building that has a sensor with a{' '}
                        <text className='font-bold underline'>POWER</text>
                        signal, that signal will be used as consumption context. It&apos;s
                        historical data will be inserted into a prediction model.
                    </p>
                    <p>
                        Make sure to add the <text className='font-bold underline'>POWER</text>{' '}
                        signal to most of the sensors linked to a building.
                    </p>
                    <Image
                        alt='3.'
                        src='/documentation/create_sensor/3.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        4. Edit the units, prefixes and aliases and click Create
                    </h3>
                    <p>
                        {' '}
                        The sensor signals can be customized. The purpose of these customizations is
                        to define a contract for the data of that sensor. Since we cannot predict
                        the format that the client will use to push data into our digital twin, this
                        format is defined here.
                    </p>
                    <br />
                    <p>
                        {' '}
                        Note that a sensor can only have{' '}
                        <text className='font-bold underline'>one</text>
                        instance of each signal.
                    </p>
                    <br />
                    <p>
                        {' '}
                        All values of the signals will be transformed to it&apos;s corresponding{' '}
                        <a
                            href='https://www.nist.gov/pml/owm/metric-si/si-units'
                            className='underline'
                        >
                            SI unit
                        </a>
                        .
                    </p>
                    <List className='text-black'>
                        <ListItem>
                            <text className='font-bold underline'>SIGNAL</text>
                            represents the type of signal.
                        </ListItem>
                        <ListItem>
                            <text className='font-bold underline'>UNIT</text>
                            indicates in which unit the signal&apos;s data will be provided.
                        </ListItem>
                        <ListItem>
                            <text className='font-bold underline'>PREFIX</text>
                            is used as a multiplication factor. For example, DECA tells our system
                            that the incoming data is multiplied with a factor 10.
                        </ListItem>
                        <ListItem>
                            <text className='font-bold underline'>ALIAS</text>
                            is the string representation of this signal. If the client would
                            provide/upload sensor data into the system, this would be the name of
                            the column representing that signal.
                        </ListItem>
                    </List>
                    <Image
                        alt='4.'
                        src='/documentation/create_sensor/4.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        5. Once the building sensor is created, it will appear on the right.
                    </h3>
                    <Image
                        alt='5'
                        src='/documentation/create_sensor/5.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                <h1
                    ref={refs.createGlobalSensor}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Create a global sensor from the sensors tab.
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>1. Click on Create Sensor</h3>
                    <p>
                        {' '}
                        There can only be <text className='font-bold underline'>one</text> global
                        sensor per twin.
                    </p>
                    <Image
                        alt='6'
                        src='/documentation/create_sensor/6.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        2. Fill in the name, description and click Next.
                    </h3>
                    <Image
                        alt='7'
                        src='/documentation/create_sensor/7.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        3. Add the desired signals and click Next.
                    </h3>
                    <p>
                        {' '}
                        A global sensor is used for context in simulations. Most of the time this
                        sensor will provide weather data to these simulators.
                    </p>
                    <br />
                    <p>
                        {' '}
                        If the client would like to create an{' '}
                        <text className='font-bold underline'>energy supply and demand</text>{' '}
                        simulation, there are some required signals:
                    </p>
                    <List className='text-black'>
                        <ListItem>
                            <text className='font-bold underline'>Irradiance</text>.
                        </ListItem>
                        <ListItem>
                            <text className='font-bold underline'>Rainfall</text>.
                        </ListItem>
                        <ListItem>
                            <text className='font-bold underline'>Temperature</text>.
                        </ListItem>
                        <ListItem>
                            <text className='font-bold underline'>Wind direction</text>.
                        </ListItem>
                        <ListItem>
                            <text className='font-bold underline'>Wind speed</text>.
                        </ListItem>
                    </List>
                    <Image
                        alt='8'
                        src='/documentation/create_sensor/8.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        4. Edit the units, prefixes and aliases and click Create
                    </h3>
                    <Image
                        alt='9.'
                        src='/documentation/create_sensor/9.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        5. Once the global sensor is created, it will appear in the list of sensors.
                        Click on the sensor to open the signal overview.
                    </h3>
                    <Image
                        alt='10'
                        src='/documentation/create_sensor/10.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        6. The signal overview shows all the signals of the sensor
                    </h3>
                    <Image
                        alt='11'
                        src='/documentation/create_sensor/11.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                {/*updateSensor*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.updateSensor}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    In the signal overview click on Update Sensor
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>1. Click on Update Sensor</h3>
                    <Image
                        alt='1'
                        src='/documentation/update_sensor/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        2. Fill in the name, description and click Next.
                    </h3>
                    <Image
                        alt='2'
                        src='/documentation/update_sensor/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        3. Add or delete the desired signals and click Next.
                    </h3>
                    <Image
                        alt='3'
                        src='/documentation/update_sensor/3.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        4. Edit the units, prefixes and aliases and click Update
                    </h3>
                    <Image
                        alt='4.'
                        src='/documentation/update_sensor/4.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        5. You can see the changes in the sensor overview and signal overview
                    </h3>
                    <Image
                        alt='5'
                        src='/documentation/update_sensor/5.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                {/*deleteSensor*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.deleteSensor}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Deleting Sensors.
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>
                        1. Use the checkboxes to select the sensors you want to delete and click
                        Delete selected sensors.
                    </h3>
                    <p>
                        {' '}
                        Once a sensor is deleted, <text className='font-bold underline'>
                            all
                        </text>{' '}
                        data of this sensor is removed from the digital twin! There is no undo
                        option.
                    </p>
                    <Image
                        alt='1'
                        src='/documentation/delete_sensor/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>2. Click on Delete</h3>
                    <Image
                        alt='2'
                        src='/documentation/delete_sensor/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                {/*createSimulation*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.createSimulation}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Create a simulation
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>1. Click on Create Simulation</h3>
                    <Image
                        alt='1'
                        src='/documentation/create_simulation/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        2. Fill in the criteria and click Next.
                    </h3>
                    <p>
                        When creating a new simulation, ensure that you provide a unique name. The
                        user should provide a timeframe to the simulator. Based on the timeframe and
                        the given timestep delta the amount timesteps for the simulation will be
                        derived.
                    </p>
                    <Image
                        alt='2'
                        src='/documentation/create_simulation/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        3. Select the simulators for the simulation and click Next.
                    </h3>
                    <p>
                        This page provides an overview of the types of simulators the user can run.
                        On the right, the components associated with each simulator are displayed.
                    </p>
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>4. Adjust components if needed.</h3>
                    <p>This page gives the user the option to adjust global components</p>
                    <Image
                        alt='4'
                        src='/documentation/create_simulation/4.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        5. Adjust the map as needed and click Create.
                    </h3>
                    <p>Read about using presets in step 6.</p>
                    <Image
                        alt='5'
                        src='/documentation/create_simulation/5.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <h3 className='text-lg font-semibold'>6. using presets:</h3>
                <div>
                    <h4>If the preset is a node: click on the preset and click on a building.</h4>
                    <Image
                        alt='4'
                        src='/documentation/create_preset/4.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h4>
                        If the preset is an edge: click on the preset and click on two buildings to
                        connect them.
                    </h4>
                    <Image
                        alt='5'
                        src='/documentation/create_preset/5.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                {/*openSimulation*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.openSimulation}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Opening Simulations.
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>1. Click on the simulation</h3>
                    <Image
                        alt='1'
                        src='/documentation/open_simulation/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        2. Here you can see the frames of the simulation.
                    </h3>
                    <Image
                        alt='2'
                        src='/documentation/open_simulation/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        3. If you click on the player settings, you can change the time between
                        frames and buffer size
                    </h3>
                    <Image
                        alt='3'
                        src='/documentation/open_simulation/3.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3>
                        4. When pressing one of the components more detailed info on the component
                        is shown.
                    </h3>
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        5. You can create a new simulation from a specific frame
                    </h3>
                    <Image
                        alt='4'
                        src='/documentation/open_simulation/4.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>
                        6. the process of creating a simulation from a frame is the same as creating
                        a normal simulation
                    </h3>
                    <Image
                        alt='5'
                        src='/documentation/open_simulation/5.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>

                {/*deleteSimulation*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.deleteSimulation}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Deleting Simulations.
                </h1>
                <div>
                    <h3 className='text-lg font-semibold'>
                        1. Use the checkboxes to select the simulations you want to delete and click
                        Delete selected simulations.
                    </h3>
                    <p>
                        {' '}
                        Once a simulation is deleted,{' '}
                        <text className='font-bold underline'>all</text> simulation frames are
                        removed from the digital twin! There is no undo option.
                    </p>
                    <Image
                        alt='1'
                        src='/documentation/delete_simulation/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>2. Click on Delete</h3>
                    <Image
                        alt='2'
                        src='/documentation/delete_simulation/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                {/*time simulator*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1 ref={refs.time} className='text-3xl md:text-4xl font-bold text-gray-800 my-4'>
                    Time simulator.
                </h1>
                <p>
                    {''}
                    This simulator is responsible for simulating the passage of time. Every frame,
                    the simulator increases the global time component by the frame duration. This
                    component is later used by the weather simulator to accurately simulate the
                    weather.
                </p>
                <h2 className='text-lg font-semibold'>Components</h2>
                <List className='text-black'>
                    <ListItem>
                        <text className='font-bold underline'>
                            TimeComponent (required/output):
                        </text>
                        <p>A component to track the time and date in the simulation.</p>
                    </ListItem>
                </List>
                {/*weather simulator*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.weather}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Weather simulator.
                </h1>
                <p>
                    {''}
                    This simulator is responsible for simulating the weather in the digital twin.
                    This simulators output can then later be used to analyse the impact of the
                    weather on different parts of the energy production, consumption and
                    distribution.
                </p>
                <p>
                    {''}
                    The weather simulator makes use of weather data captured by the registered
                    global sensor. This data is then used by a prediction model in order to predict
                    the weather variables for the current frame.
                </p>
                <h2 className='text-lg font-semibold'>Components</h2>
                <List className='text-black'>
                    <ListItem>
                        <text className='font-bold underline'>TimeComponent (required):</text>
                        <p> A component to track the time and date in the simulation. </p>
                    </ListItem>
                    <ListItem>
                        <text className='font-bold underline'>
                            TemperatureComponent (optional/output):
                        </text>
                        <p> A component that contains the temperature in degrees Celsius. </p>
                    </ListItem>
                    <ListItem>
                        <text className='font-bold underline'>
                            PrecipitationComponent (optional/output):
                        </text>
                        <p> A component that contains the amount of precipitation in mm/hour. </p>
                    </ListItem>
                    <ListItem>
                        <text className='font-bold underline'>
                            WindSpeedComponent (optional/output):
                        </text>
                        <p> A component representing the wind speed in m/s. </p>
                    </ListItem>
                    <ListItem>
                        <text className='font-bold underline'>
                            WindDirectionComponent (optional/output):
                        </text>
                        <p> A component to represent the wind direction in degrees. </p>
                    </ListItem>
                    <ListItem>
                        <text className='font-bold underline'>
                            IrradianceComponent (optional/output):
                        </text>
                        <p> A component representing solar irradiance in W/m^2. </p>
                    </ListItem>
                </List>
                {/*supply and demand simulator*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.supplyAndDemand}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Energy supply and demand simulator.
                </h1>
                <p>
                    {''}
                    This simulator is responsible for simulating the energy supply and demand. Both
                    energy supply and energy demand should vary with date and time, and they are
                    also affected by weather variables. For example, the energy required for
                    household heating will increase as the outdoor temperature decreases. Renewable
                    energy sources such as solar panels and wind turbines produce different amounts
                    of energy based on weather variables such as solar irradiance and wind speed.
                </p>
                <h2 className='text-lg font-semibold'>Components</h2>
                <List className='text-black'>
                    <ListItem>
                        <text className='font-bold underline'>TimeComponent (required):</text>
                        <p> A component to track the time and date in the simulation. </p>
                    </ListItem>
                    <ListItem>
                        <text className='font-bold underline'>Building (required):</text>
                        <p>
                            {' '}
                            A component to represent a building. The components contain a building
                            id, that corresponds to a building in the digital twin. Buildings are
                            usually associated with an energy consumer and/or energy producer.
                        </p>
                    </ListItem>
                    <ListItem>
                        <text className='font-bold underline'>
                            SensorLoadNode (required/output):
                        </text>
                        <p>
                            {' '}
                            A component that represents an entity that demands energy. The demanded
                            energy is expressed in watts.
                        </p>
                    </ListItem>
                    <ListItem>
                        <text className='font-bold underline'>
                            SensorGeneratorNode (required/output):
                        </text>
                        <p>
                            {' '}
                            A component that represents an entity that produces energy. The produced
                            energy is expressed in watts. The type1 of the energy source should be
                            specified as well.
                        </p>
                    </ListItem>
                    <ListItem>
                        <text className='font-bold underline'>WindSpeedComponent (optional):</text>
                        <p> A component representing the wind speed in m/s. </p>
                    </ListItem>
                    <ListItem>
                        <text className='font-bold underline'>IrradianceComponent (optional):</text>
                        <p> A component representing solar irradiance in W/m^2. </p>
                    </ListItem>
                    <ListItem>
                        <text className='font-bold underline'>
                            SupplyAndDemandAnalytics (optional/output):
                        </text>
                        <p>
                            A component that contains some analytics about the energy supply and
                            demand simulation. For example the number of energy consumer and
                            producers, the total demand, ... This component can also contain an
                            overview of the different types1 of energy sources present in the
                            simulation and their respective contribution (expressed in percentage)
                            to the total produced energy.
                        </p>
                    </ListItem>
                </List>
                {/*loadflow simulator*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.loadflow}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Loadflow simulator.
                </h1>
                <h5 className='font-bold underline'>
                    Requirements for running load-flow simulation:
                </h5>
                <p>
                    {' '}
                    This simulator is dependent on the sensor data provided to give an estimate of
                    the initial power configuration for the consumer and producer nodes in the
                    network. If sensors are not placed on all nodes, the initial configuration may
                    not be accurate for the load-flow simulator.
                </p>
                <h4>
                    <text className='font-bold underline'>Common Practices:</text>
                </h4>

                <List nested className='text-black'>
                    <List.Item>
                        Try using at least three components for a simulation. Since the underlying
                        solver uses Newton-Raphson, this will increase the conversion rate.
                    </List.Item>
                    <List.Item>
                        Ensure that the graph is connected, with no nodes completely disconnected
                        from the graph. Disconnected nodes can lead to many zeros in the admittance
                        matrix, preventing the simulation solver from running optimally.
                    </List.Item>
                </List>
                <h4>
                    <text className='font-bold underline'>How to Interpret the Results:</text>
                </h4>

                <List nested className='text-black'>
                    <List.Item>
                        The load-flow simulator will attempt to find a realistic configuration for
                        the provided graph by adjusting the voltages on the nodes. There are three
                        possible scenarios:
                    </List.Item>
                    <List ordered nested className='text-black'>
                        <List.Item>
                            The current and voltage (angle) change. The current updates according to
                            the updated voltages. The transmission lines remain green indicating the
                            calculated voltages and currents are within range.
                        </List.Item>
                        <List.Item>
                            The new voltages become exponentially large. This could be due to the
                            user not providing a good estimated initial configuration. The solver
                            may increase the voltages without converging to valid data, resulting in
                            the lines turning red due to the user-specified maximum acceptable
                            current flow.
                        </List.Item>
                        <List.Item>
                            The voltages reset to fixed values. This can happen if the initial
                            configuration was not provided correctly. The solver may reset the
                            values to voltage (1.0, 0.0) for amplitude and angle and require the
                            user to provide a new initial configuration.
                        </List.Item>
                    </List>
                </List>
                {/*Simulator components*/}
                <hr className='my-2 border-t border-gray-300 pb-5' />
                <h1
                    ref={refs.components}
                    className='text-3xl md:text-4xl font-bold text-gray-800 my-4'
                >
                    Graph components.
                </h1>
                <List className='text-black'>
                    <List.Item>
                        <text className='font-bold underline'>energy_generator_node</text>
                        <div>The possible entries for energy production types are:</div>
                        <List nested className='text-black'>
                            <List.Item>Solar</List.Item>
                            <List.Item>Storage</List.Item>
                            <List.Item>Battery</List.Item>
                            <List.Item>Wind</List.Item>
                            <List.Item>Hydro</List.Item>
                            <List.Item>Fossil</List.Item>
                            <List.Item>Renewable</List.Item>
                            <List.Item>Nuclear</List.Item>
                        </List>
                    </List.Item>
                    <List.Item>
                        <text className='font-bold underline'>energy_transmission_edge</text>
                        <div>
                            The transmission edge component requires the user to provide a cable
                            type. Cable type options are:
                        </div>
                        <List nested className='text-black'>
                            <List.Item>ACSR_Conductor</List.Item>
                            <List.Item>AAC_Conductor</List.Item>
                            <List.Item>AAAC_Conductor</List.Item>
                            <List.Item>XLPE_Conductor</List.Item>
                            <List.Item>PILC_Conductor</List.Item>
                        </List>
                    </List.Item>
                </List>
            </div>
        </div>
    );
};

export default ContentSection;
