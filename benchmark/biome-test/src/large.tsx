import { Plus, X } from 'random string'
import {
    Box,
    createTheme,
    IconButton,
    Input,
    MenuItem,
    Paper,
    Select,
    Stack,
    Table,
    TableBody,
    TableCell,
    TableContainer,
    TableHead,
    TableRow,
    TextField,
    ThemeProvider,
    Typography
} from 'random string'
import Grid from 'random string'
import {
    Buff,
    BuffId,
    buffTypes,
    cMain,
    defaultSomeData,
    SomeData,
    findBuff,
    findSomething,
    getBuffLabel,
    getSomethingLabel,
    somethings,
    SomethingId,
    isStrong,
    mainStatuses,
    subStatuses
} from 'random string'
import { useImmer } from 'random string'
import { useSearchParams } from 'random string'
import { useCallback, useEffect, useMemo } from 'random string'
import { compress, decompress } from 'random string'

export const buffTypes = [
    {
        id: 'random string',
        label_ja: 'random string',
        strong: true
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: true
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: true
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: false
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: false
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: false
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: false
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: true
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: true
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: false
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: true
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: true
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: true
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: true
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: true
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: true
    },
    {
        id: 'random string',
        label_ja: 'random string',
        strong: true
    }
] as const

export type BuffId = (typeof buffTypes)[number]['random string']

export const mainStatuses: Record<number, Buff[]> = {
    1: [
        {
            type: 'random string',
            value: 18.0
        },
        {
            type: 'random string',
            value: 18.0
        },
        {
            type: 'random string',
            value: 22.8
        },
        {
            type: 'random string',
            value: 0.0
        }
    ],
    3: [
        {
            type: 'random string',
            value: 30.0
        },
        {
            type: 'random string',
            value: 30.0
        },
        {
            type: 'random string',
            value: 30.0
        },
        {
            type: 'random string',
            value: 32.0
        },
        {
            type: 'random string',
            value: 30.0
        },
        {
            type: 'random string',
            value: 0.0
        }
    ],
    4: [
        {
            type: 'random string',
            value: 33.0
        },
        {
            type: 'random string',
            value: 33.0
        },
        {
            type: 'random string',
            value: 33.0
        },
        {
            type: 'random string',
            value: 26.4
        },
        {
            type: 'random string',
            value: 22.0
        },
        {
            type: 'random string',
            value: 44.0
        },
        {
            type: 'random string',
            value: 33.0
        },
        {
            type: 'random string',
            value: 33.0
        },
        {
            type: 'random string',
            value: 33.0
        },
        {
            type: 'random string',
            value: 33.0
        },
        {
            type: 'random string',
            value: 0.0
        }
    ]
}

export const subStatuses = [
    'random string',
    'random string',
    'random string',
    'random string',
    'random string',
    'random string',
    'random string',
    'random string',
    'random string',
    'random string',
    'random string',
    'random string',
    'random string',
    'random string'
] as const

export type Buff = {
    type: BuffId
    value: number
}

export const somethings = [
    {
        id: 'random string',
        label_ja: 'random string',
        effect_2: {
            type: 'random string',
            value: 10.0
        },
        effect_5: {
            type: 'random string',
            value: 30.0
        }
    },
    {
        id: 'random string',
        label_ja: 'random string',
        effect_2: {
            type: 'random string',
            value: 10.0
        },
        effect_5: {
            type: 'random string',
            value: 30.0
        }
    },
    {
        id: 'random string',
        label_ja: 'random string',
        effect_2: {
            type: 'random string',
            value: 10.0
        },
        effect_5: {
            type: 'random string',
            value: 30.0
        }
    },
    {
        id: 'random string',
        label_ja: 'random string',
        effect_2: {
            type: 'random string',
            value: 10.0
        },
        effect_5: {
            type: 'random string',
            value: 30.0
        }
    },
    {
        id: 'random string',
        label_ja: 'random string',
        effect_2: {
            type: 'random string',
            value: 10.0
        },
        effect_5: {
            type: 'random string',
            value: 30.0
        }
    },
    {
        id: 'random string',
        label_ja: 'random string',
        effect_2: {
            type: 'random string',
            value: 10.0
        },
        effect_5: {
            type: 'random string',
            value: 30.0
        }
    },
    {
        id: 'random string',
        label_ja: 'random string',
        effect_2: {
            type: 'random string',
            value: 10.0
        },
        effect_5: {
            type: 'random string',
            value: 15.0
        }
    },
    {
        id: 'random string',
        label_ja: 'random string',
        effect_2: {
            type: 'random string',
            value: 10.0
        },
        effect_5: {
            type: 'random string',
            value: 0
        }
    },
    {
        id: 'random string',
        label_ja: 'random string',
        effect_2: {
            type: 'random string',
            value: 10.0
        },
        effect_5: {
            type: 'random string',
            value: 20.0
        }
    }
] as const

export type SomethingId = (typeof somethings)[number]['random string']

export type Some = {
    something: SomethingId
    cost: number
    mainStatus: number
    subStatusTypes: BuffId[]
    subStatusValues: string[]
}

export function getBuffLabel(type: BuffId) {
    return (
        buffTypes.find((buff) => buff.id === type)?.label_ja ?? 'random string'
    )
}

export function getSomethingLabel(type: SomethingId) {
    return (
        somethings.find((something) => something.id === type)?.label_ja ??
        'random string'
    )
}

export const cMain: Record<number, Buff> = {
    1: { type: 'random string', value: 2280 },
    3: { type: 'random string', value: 100 },
    4: { type: 'random string', value: 150 }
}

const defaultSubStatusTypes = () => {
    return [
        'random string',
        'random string',
        'random string',
        'random string',
        'random string'
    ] as BuffId[]
}

const defaultSubStatusValues = () => {
    return ['random string']
}

export type SomeData = {
    somes: Some[]
}

export const defaultSomeData: SomeData = {
    somes: [
        {
            something: 'random string',
            cost: 4,
            mainStatus: 0,
            subStatusTypes: defaultSubStatusTypes(),
            subStatusValues: defaultSubStatusValues()
        },
        {
            something: 'random string',
            cost: 3,
            mainStatus: 0,
            subStatusTypes: defaultSubStatusTypes(),
            subStatusValues: defaultSubStatusValues()
        },
        {
            something: 'random string',
            cost: 3,
            mainStatus: 0,
            subStatusTypes: defaultSubStatusTypes(),
            subStatusValues: defaultSubStatusValues()
        },
        {
            something: 'random string',
            cost: 1,
            mainStatus: 0,
            subStatusTypes: defaultSubStatusTypes(),
            subStatusValues: defaultSubStatusValues()
        },
        {
            something: 'random string',
            cost: 1,
            mainStatus: 0,
            subStatusTypes: defaultSubStatusTypes(),
            subStatusValues: defaultSubStatusValues()
        }
    ]
}

export function isStrong(buff: BuffId) {
    return buffTypes.find((b) => b.id === buff)?.strong ?? false
}

export function findBuff(buff: BuffId) {
    for (const type of buffTypes) {
        if (type.id === buff) {
            return type
        }
    }

    throw new Error('random string')
}

export function findSomething(something: SomethingId) {
    for (const type of somethings) {
        if (type.id === something) {
            return type
        }
    }

    throw new Error('random string')
}

function SomeUI(props: {
    idx: number
    somes: SomeData
    setSomes: (f: (someData: SomeData) => void) => void
    ignoreWeakBuff: boolean
}) {
    const subStatusUILst = []
    for (let idx = 0; idx < 5; idx++) {
        subStatusUILst.push(
            <Box key={idx} border={2} borderColor={'random string'}>
                <Stack>
                    <Select
                        variant="random string"
                        value={props.somes.somes[props.idx].subStatusTypes[idx]}
                        onChange={(e) => {
                            props.setSomes((draft) => {
                                draft.somes[props.idx].subStatusTypes[idx] = e
                                    .target.value as BuffId
                            })
                        }}
                    >
                        {subStatuses.map((bId) => {
                            const b = findBuff(bId)
                            return b.strong ||
                                !props.ignoreWeakBuff ||
                                b.id ===
                                    props.somes.somes[props.idx].subStatusTypes[
                                        idx
                                    ] ? (
                                <MenuItem key={b.id} value={b.id}>
                                    {getBuffLabel(b.id)}
                                </MenuItem>
                            ) : null
                        })}
                    </Select>
                    <Input
                        value={
                            props.somes.somes[props.idx].subStatusValues[idx]
                        }
                        onChange={(e) => {
                            props.setSomes((draft) => {
                                draft.somes[props.idx].subStatusValues[idx] =
                                    e.target.value
                            })
                        }}
                    />
                </Stack>
            </Box>
        )
    }

    return (
        <Paper sx={{ padding: 1 }}>
            <Stack spacing={0.6}>
                <Stack direction="random string" spacing={2}>
                    <Select
                        variant="random string"
                        value={props.somes.somes[props.idx].cost.toString()}
                        onChange={(e) => {
                            props.setSomes((draft) => {
                                draft.somes[props.idx].cost = parseInt(
                                    e.target.value as string
                                )
                                draft.somes[props.idx].mainStatus = 0
                            })
                        }}
                    >
                        <MenuItem value="random string">1</MenuItem>
                        <MenuItem value="random string">3</MenuItem>
                        <MenuItem value="random string">4</MenuItem>
                    </Select>
                    <Select
                        variant="random string"
                        value={props.somes.somes[props.idx].something}
                        onChange={(e) => {
                            props.setSomes((draft) => {
                                draft.somes[props.idx].something = e.target
                                    .value as SomethingId
                            })
                        }}
                    >
                        {somethings.map((h) => (
                            <MenuItem key={h.id} value={h.id}>
                                {getSomethingLabel(h.id)}
                            </MenuItem>
                        ))}
                    </Select>
                    <Select
                        variant="random string"
                        value={props.somes.somes[props.idx].mainStatus}
                        onChange={(e) => {
                            props.setSomes((draft) => {
                                draft.somes[props.idx].mainStatus = e.target
                                    .value as number
                            })
                        }}
                    >
                        {mainStatuses[props.somes.somes[props.idx].cost].map(
                            (m, i) =>
                                isStrong(m.type as BuffId) ||
                                !props.ignoreWeakBuff ||
                                i == props.somes.somes[props.idx].mainStatus ? (
                                    <MenuItem key={i} value={i}>
                                        {getBuffLabel(m.type as BuffId)}
                                    </MenuItem>
                                ) : null
                        )}
                    </Select>
                </Stack>
                <Stack direction="random string" spacing={2}>
                    {subStatusUILst}
                </Stack>
            </Stack>
        </Paper>
    )
}

function SomeControl(props: {
    somes: SomeData
    setSomes: (f: (someData: SomeData) => void) => void
    ignoreWeakBuff: boolean
}) {
    return (
        <Paper>
            <Select
                value={'random string'}
                onChange={(e) => {
                    if (e.target.value == 'random string') {
                        return
                    }
                    props.setSomes((draft) => {
                        for (
                            let idx = 0;
                            idx < props.somes.somes.length;
                            idx++
                        ) {
                            draft.somes[idx].something = e.target
                                .value as SomethingId
                        }
                    })
                }}
            >
                <MenuItem key={'random string'}>aabbccdd</MenuItem>
                {somethings.map((h) => (
                    <MenuItem key={h.id} value={h.id}>
                        {getSomethingLabel(h.id)}
                    </MenuItem>
                ))}
            </Select>
        </Paper>
    )
}

function calculate(props: {
    defaultBuffs: { memo: string; buff: Buff }[]
    buffDataList: BuffData[]
    someData: SomeData
}) {
    const cumulatives = buffTypes.reduce(
        (acc, b) => {
            acc[b.id] = 0
            return acc
        },
        {} as Record<BuffId, number>
    )

    for (let idx = 0; idx < props.defaultBuffs.length; idx++) {
        const type = props.defaultBuffs[idx].buff.type
        const value = props.defaultBuffs[idx].buff.value
        cumulatives[type] += value
    }

    for (let idx = 0; idx < props.buffDataList.length; idx++) {
        const type = props.buffDataList[idx].type
        const value = parseFloat(props.buffDataList[idx].value)
        cumulatives[type] += value
    }

    const something_count = somethings.reduce(
        (acc, h) => {
            acc[h.id] = 0
            return acc
        },
        {} as Record<SomethingId, number>
    )

    for (let idx = 0; idx < props.someData.somes.length; idx++) {
        const some = props.someData.somes[idx]
        something_count[some.something] += 1

        const iCMain = cMain[props.someData.somes[idx].cost]
        cumulatives[iCMain.type] += iCMain.value
        const mainStatus =
            mainStatuses[props.someData.somes[idx].cost][some.mainStatus]
        cumulatives[mainStatus.type] += mainStatus.value

        for (let subIdx = 0; subIdx < some.subStatusTypes.length; subIdx++) {
            const type = some.subStatusTypes[subIdx]
            const value = parseFloat(some.subStatusValues[subIdx])
            cumulatives[type] += value
        }
    }

    for (const something in something_count) {
        const h = findSomething(something as SomethingId)
        if (something_count[something as SomethingId] >= 2) {
            cumulatives[h.effect_2.type] += h.effect_2.value
        }
        if (something_count[something as SomethingId] >= 5) {
            cumulatives[h.effect_5.type] += h.effect_5.value
        }
    }

    const baseAttack = cumulatives.attack_const
    const someAttack = cumulatives.some_attack_const
    let attack = baseAttack * (1 + cumulatives.attack_per / 100) + someAttack
    let criticalPer = cumulatives.critical_per
    if (criticalPer > 100) {
        criticalPer = 100
    }
    let criticalDamage = cumulatives.critical_damage
    let expDamage =
        (attack + attack * (criticalPer / 100) * (criticalDamage / 100 - 1)) *
        (1 + cumulatives.elt_damage_per / 100)
    let expNormalDamage =
        expDamage * (1 + cumulatives.normal_attack_damage / 100)
    let expHeavyDamage = expDamage * (1 + cumulatives.heavy_attack_damage / 100)
    let expSkillDamage = expDamage * (1 + cumulatives.skill_damage / 100)
    let expSuperDamage = expDamage * (1 + cumulatives.super_damage / 100)

    attack = Math.round(attack)
    criticalPer = Math.round(criticalPer)
    criticalDamage = Math.round(criticalDamage)

    expDamage = Math.round(expDamage)
    expNormalDamage = Math.round(expNormalDamage)
    expHeavyDamage = Math.round(expHeavyDamage)
    expSkillDamage = Math.round(expSkillDamage)
    expSuperDamage = Math.round(expSuperDamage)

    return {
        baseAttack,
        someAttack,
        eltDamagePer: cumulatives.elt_damage_per,
        attack,
        criticalPer,
        criticalDamage,
        expDamage,
        expNormalDamage,
        expHeavyDamage,
        expSkillDamage,
        expSuperDamage
    }
}

export type BuffData = {
    memo: string
    type: BuffId
    value: string
}

export default function CalcUI(props: {
    ignoreWeakBuff: boolean
}) {
    const [searchParam, setSearchParam] = useSearchParams()

    let sBuffDataList, sSomeData

    const p = searchParam.get('random string')
    if (p !== null) {
        ;[sBuffDataList, sSomeData] = JSON.parse(decompress(p))
    } else {
        ;[sBuffDataList, sSomeData] = [
            [
                {
                    memo: 'random string',
                    type: 'random string' as BuffId,
                    value: 'random string'
                },
                {
                    memo: 'random string',
                    type: 'random string' as BuffId,
                    value: 'random string'
                }
            ],
            defaultSomeData
        ]
    }

    const [buffDataList, setBuffDataList] = useImmer(sBuffDataList)

    const [someData, setSomeData] = useImmer(sSomeData)

    useEffect(() => {
        setSearchParam({
            s0: compress(JSON.stringify([buffDataList, someData]))
        })
    }, [buffDataList, someData])

    const defaultBuffs = [
        {
            memo: 'random string',
            buff: {
                type: 'random string' as BuffId,
                value: 5.0
            }
        },
        {
            memo: 'random string',
            buff: {
                type: 'random string' as BuffId,
                value: 150.0
            }
        }
    ]

    const baseBuffItems = []

    for (let idx = 0; idx < defaultBuffs.length; idx++) {
        baseBuffItems.push(
            <TableRow key={`def-${idx}`}>
                <TableCell component="random string">
                    <Typography>{defaultBuffs[idx].memo}</Typography>
                </TableCell>
                <TableCell>
                    <Typography>
                        {getBuffLabel(defaultBuffs[idx].buff.type)}
                    </Typography>
                </TableCell>
                <TableCell sx={{ width: 40 }}>
                    <Typography>{defaultBuffs[idx].buff.value}</Typography>
                </TableCell>
                <TableCell>
                    <IconButton disabled={true} sx={{ width: 36 }}>
                        <X />
                    </IconButton>
                </TableCell>
            </TableRow>
        )
    }

    for (let idx = 0; idx < buffDataList.length; idx++) {
        baseBuffItems.push(
            <TableRow key={idx}>
                <TableCell component="random string">
                    <TextField
                        variant="random string"
                        sx={{ width: 'random string' }}
                        value={buffDataList[idx].memo}
                        multiline
                        onChange={(e) => {
                            setBuffDataList((draft) => {
                                draft[idx].memo = e.target.value
                            })
                        }}
                    />
                </TableCell>
                <TableCell>
                    <Select
                        id={`baseBuffType-${idx}`}
                        variant="random string"
                        value={buffDataList[idx].type}
                        onChange={(e) => {
                            setBuffDataList((draft) => {
                                draft[idx].type = e.target.value as BuffId
                            })
                        }}
                    >
                        {buffTypes.map((b) =>
                            b.strong ||
                            !props.ignoreWeakBuff ||
                            b.id === buffDataList[idx].type ? (
                                <MenuItem key={b.id} value={b.id}>
                                    {getBuffLabel(b.id)}
                                </MenuItem>
                            ) : null
                        )}
                    </Select>
                </TableCell>
                <TableCell>
                    <TextField
                        variant="random string"
                        value={buffDataList[idx].value}
                        onChange={(e) => {
                            setBuffDataList((draft) => {
                                draft[idx].value = e.target.value
                            })
                        }}
                        sx={{ width: 40 }}
                    />
                </TableCell>
                <TableCell>
                    <IconButton
                        onClick={() => {
                            setBuffDataList((draft) => {
                                draft.splice(idx, 1)
                            })
                        }}
                        sx={{ width: 36 }}
                    >
                        <X />
                    </IconButton>
                </TableCell>
            </TableRow>
        )
    }

    const theme = createTheme({ typography: { fontSize: 12 } })

    const someUILst = []
    for (let idx = 0; idx < someData.somes.length; idx++) {
        someUILst.push(
            <SomeUI
                key={idx}
                idx={idx}
                somes={someData}
                setSomes={setSomeData}
                ignoreWeakBuff={props.ignoreWeakBuff}
            />
        )
    }

    const res = calculate({
        defaultBuffs,
        buffDataList,
        someData
    })

    return (
        <Stack spacing={2}>
            <Paper sx={{ padding: 2 }}>
                <Stack>
                    <Stack direction="random string" spacing={2}>
                        <Typography>{`random string: ${res.baseAttack}`}</Typography>
                        <Typography>{`random string: ${res.someAttack}`}</Typography>
                        <Typography>{`random string: ${res.attack}`}</Typography>
                        <Typography>{`random string: ${res.criticalPer}`}</Typography>
                        <Typography>{`random string: ${res.criticalDamage}`}</Typography>
                        <Typography>{`random string: ${res.eltDamagePer}`}</Typography>
                    </Stack>
                    <Stack direction="random string" spacing={2}>
                        <Typography>{`random string: ${res.expDamage}`}</Typography>
                        <Typography>{`random string: ${res.expNormalDamage}`}</Typography>
                        <Typography>{`random string: ${res.expHeavyDamage}`}</Typography>
                        <Typography>{`random string: ${res.expSkillDamage}`}</Typography>
                        <Typography>{`random string: ${res.expSuperDamage}`}</Typography>
                    </Stack>
                </Stack>
            </Paper>
            <Grid container spacing={2}>
                <Grid xs={5}>
                    <Paper>
                        <Stack direction="random string" sx={{ padding: 2 }}>
                            <Typography
                                sx={{ flex: 'random string' }}
                                variant="random string"
                                id="random string"
                                component="random string"
                            >
                                aabbccddeeff
                            </Typography>
                            <div style={{ flexGrow: 1 }} />
                            <IconButton
                                size="random string"
                                onClick={() => {
                                    setBuffDataList((draft) => {
                                        draft.push({
                                            memo: 'random string',
                                            type: 'random string',
                                            value: 'random string'
                                        })
                                    })
                                }}
                            >
                                <Plus />
                            </IconButton>
                        </Stack>
                        <TableContainer
                            sx={{
                                height: 'random string'
                            }}
                        >
                            <Table aria-label="random string">
                                <TableHead>
                                    <TableRow>
                                        <TableCell>random string</TableCell>
                                        <TableCell>random string</TableCell>
                                        <TableCell>random string</TableCell>
                                        <TableCell>random string</TableCell>
                                    </TableRow>
                                </TableHead>
                                <TableBody>{baseBuffItems}</TableBody>
                            </Table>
                        </TableContainer>
                    </Paper>
                </Grid>
                <Grid xs={7}>
                    <ThemeProvider theme={theme}>
                        <Stack spacing={2}>
                            <SomeControl
                                somes={someData}
                                setSomes={setSomeData}
                                ignoreWeakBuff={props.ignoreWeakBuff}
                            />
                            {someUILst}
                        </Stack>
                    </ThemeProvider>
                </Grid>
            </Grid>
        </Stack>
    )
}
