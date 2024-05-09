import React from 'react';
import Image from 'next/image';

interface SectionRefs {
    [key: string]: React.RefObject<HTMLDivElement>;
}

interface ContentSectionProps {
    refs: SectionRefs;
}

const ContentSection: React.FC<ContentSectionProps> = ({ refs }) => {
    return (
        <div className='flex flex-col items-center justify-center'>
            <div className={'w-[1059px] space-y-10'}>
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
                        If you are not logged in yet you will see two buttons that say
                        &quot;Register&quot; and &quot;Login&quot;. Click on Register to continue.
                        Otherwise you will see a Button that says &quot;Dashboard&quot;. Once you
                        click on Dashboard you will be redirected to the dashboard page and leave
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
                        4. Follow the instructions in the modal and click login.
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
                    <h3 className='text-lg font-semibold'>2. Click on Create twin</h3>
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
                        3. Choose the city, place or choose coordinates and click search. (you
                        don&apos;t have to choose coordinates if you choose a place and vice versa.)
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
                        to delete and click delete selected twins.
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
                    <h3 className='text-lg font-semibold'>2. Click on delete.</h3>
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
                    <p>TODO DESCRIPTION</p>
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
                        TODO DESCRIPTION You can&apos;t add a node and an edge to the same preset.
                        This will give a warning
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
                <h3 className='text-lg font-semibold'>4. using presets:</h3>
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
                    <p>TODO DESCRIPTION</p>
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
                        4. Edit the units, prefixes and aliases and click create
                    </h3>
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
                    <p>TODO DESCRIPTION</p>
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
                        4. Edit the units, prefixes and aliases and click create
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
                        4. Edit the units, prefixes and aliases and click create
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
                        Delete Selected Sensors.
                    </h3>
                    <Image
                        alt='1'
                        src='/documentation/delete_sensor/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>2. Click on delete</h3>
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
                    <p>TODO DESCRIPTION</p>
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
                    <p>TODO DESCRIPTION</p>
                    <Image
                        alt='3'
                        src='/documentation/create_simulation/3.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>4. Adjust components if needed.</h3>
                    <p>TODO DESCRIPTION</p>
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
                        5. Adjust the map as needed and click create.{' '}
                    </h3>
                    <Image
                        alt='5'
                        src='/documentation/create_simulation/5.png'
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
                    <h3 className='text-lg font-semibold'>
                        4. You can create a new simulation from a specific frame
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
                        5. the process of creating a simulation from a frame is the same as creating
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
                        Delete Selected Simulations.
                    </h3>
                    <Image
                        alt='1'
                        src='/documentation/delete_sensor/1.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
                <div>
                    <h3 className='text-lg font-semibold'>2. Click on delete</h3>
                    <Image
                        alt='2'
                        src='/documentation/delete_sensor/2.png'
                        style={{ borderRadius: 8, border: '1px solid #F4F2F7' }}
                        width={1059}
                        height={710}
                    />
                </div>
            </div>
        </div>
    );
};

export default ContentSection;
