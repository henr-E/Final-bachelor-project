import React, { ReactNode, useEffect, useState } from 'react';
import { Accordion, Button, TextInput } from 'flowbite-react';
import { MdOutlineDeleteOutline } from 'react-icons/md';
import { AiOutlinePlus } from 'react-icons/ai';
import { ComponentStructure } from '@/proto/simulation/simulation';

interface JsonData {
    [key: string]: any;
}

interface Props {
    data?: JsonData;
    onSave: (data: JsonData | undefined) => void; // Callback to handle save operation
}

export const TypeConverter = (structure: ComponentStructure | undefined): any => {
    if (structure?.primitive == 11) {
        return '';
    } else if (structure?.primitive && structure?.primitive <= 10) {
        return 0;
    } else if (structure?.primitive && structure?.primitive >= 12) {
        return 0.1;
    } else if (structure?.list) {
        return [TypeConverter(structure?.list)];
    } else if (structure?.struct) {
        let items: { [key: string]: any } = {};

        Object.keys(structure?.struct.data).forEach(function (key, index) {
            items[key] = TypeConverter(structure?.struct?.data[key]);
        });
        return items;
    } else {
        console.log('error');
    }
};

const CustomJsonEditor: React.FC<Props> = ({ data, onSave }) => {
    const [editableData, setEditableData] = useState<JsonData | undefined>(data);

    useEffect(() => {
        setEditableData(data);
    }, [data]);

    const handleInputChange = (originalPath: string[], value: string) => {
        if (!editableData) return;

        let newData = { ...editableData };
        let current: any = newData;
        let path = [...originalPath];
        const lastKey = path.pop();

        for (const p of path) {
            if (current[p] === undefined) {
                current[p] = {};
            }
            current = current[p];
        }

        if (lastKey !== undefined) {
            if (value === '' && typeof current[lastKey] === 'number') {
                current[lastKey] = '';
            } else {
                try {
                    current[lastKey] = JSON.parse(value);
                } catch {
                    current[lastKey] =
                        typeof current[lastKey] === 'number' ? parseFloat(value) || 0 : value;
                }
            }
        }

        setEditableData(newData);
    };

    const addItemToArray = (path: string[]) => {
        if (!editableData) return;

        let newData = { ...editableData };
        let current: any = newData;
        for (const p of path) {
            current = current[p];
        }

        // Check if the array is empty
        if (current.length === 0) {
            // Initialize the array with a default value
            current.push({ key: '', value: 0 }); // You can adjust the default values as needed
        } else {
            // Copy the first element
            current.push({ ...current[0] });
        }

        setEditableData(newData);
    };

    const deleteItemFromArray = (path: string[], index: number) => {
        if (!editableData) return;

        let newData = { ...editableData };
        let current: any = newData;
        for (const p of path) {
            current = current[p];
        }

        current.splice(index, 1);
        setEditableData(newData);
    };

    const renderField = (key: string, value: any, parentPath: string[]): ReactNode => {
        const fullPath = [...parentPath, key];
        let inputType = typeof value === 'number' ? 'number' : 'text';

        return (
            <TextInput
                type={inputType}
                value={value.toString()}
                onChange={e => handleInputChange(fullPath, e.target.value)}
            />
        );
    };

    const renderRows = (data: JsonData | undefined, prefix: string[]): ReactNode => {
        if (!data) return null;
        return Object.entries(data).map(([key, value]) => {
            const path = [...prefix, key];
            if (Array.isArray(value)) {
                return (
                    <React.Fragment key={path.join('.')}>
                        <p className='pt-2'>{key}</p>
                        {value.map((item, index) => (
                            <tr key={`${path.join('.')}.${index}`}>
                                {Object.entries(item).map(([itemKey, itemValue]) => (
                                    <td key={itemKey}>
                                        <div className='basis-1/2'>
                                            {`${itemKey}: `}
                                            {renderField(
                                                itemKey,
                                                itemValue,
                                                path.concat(index.toString())
                                            )}
                                        </div>
                                    </td>
                                ))}
                                <td>
                                    <p>Delete</p>
                                    <Button
                                        className='h-11 inline-flex items-center justify-center'
                                        onClick={() => deleteItemFromArray(path, index)}
                                        color='indigo'
                                        theme={{
                                            color: {
                                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                            },
                                        }}
                                    >
                                        <MdOutlineDeleteOutline />
                                    </Button>
                                </td>
                            </tr>
                        ))}
                        <tr>
                            <td></td>
                            <td></td>
                            <td>
                                <p>Add item</p>
                                <Button
                                    className='h-11 inline-flex items-center justify-center'
                                    onClick={() => addItemToArray(path)}
                                    color='indigo'
                                    theme={{
                                        color: {
                                            indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                        },
                                    }}
                                >
                                    <AiOutlinePlus />
                                </Button>
                            </td>
                        </tr>
                    </React.Fragment>
                );
            } else if (typeof value === 'object' && value !== null) {
                return (
                    <Accordion key={key} collapseAll>
                        <Accordion.Panel>
                            <Accordion.Title>{key}</Accordion.Title>
                            <Accordion.Content>
                                {renderRows(value, path)}
                                <div className='flex justify-center mt-4'>
                                    <Button
                                        className='w-full py-2'
                                        color='indigo'
                                        theme={{
                                            color: {
                                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                                            },
                                        }}
                                        onClick={() => onSave(editableData)}
                                    >
                                        Save
                                    </Button>
                                </div>
                            </Accordion.Content>
                        </Accordion.Panel>
                    </Accordion>
                );
            } else {
                //not an array
                return (
                    <tr key={path.join('.')} className={'w-full'}>
                        <td className={'flex'}>
                            <div className='basis-1/2'>{key}</div>
                            <div className='basis-1/2'>
                                {renderField(key, value, path.slice(0, -1))}
                            </div>
                        </td>
                    </tr>
                );
            }
        });
    };

    return (
        <div className='flex w-full'>
            <div className='w-full'>
                <table className='w-full'>
                    <tbody>{renderRows(editableData, [])}</tbody>
                </table>
            </div>
        </div>
    );
};

export default CustomJsonEditor;
