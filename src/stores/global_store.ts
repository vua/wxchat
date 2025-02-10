import {defineStore} from 'pinia'
import {ref} from "vue"
import {TableInstance} from "element-plus";

export interface Member {
    NickName: string
    RemarkName: string
    UserName: string
    PYQuanPin: string
    VerifyFlag: number
}

export interface User {
    NickName: string
    UserName: string
    PYQuanPin: string
}

export interface Group {
    Id: string
    Name: string
    Members: Member[]
    Operator: string
}

export interface Rule {
    Id: string
    Name: string
    Group: string
    Reply: Reply[]
    Status: string
}


export interface Reply {
    ReplyType: string
    Template: Template
    OpenAi: string
}

export interface Template {
    Keywords: string[]
    Content: string
    Keyword: string
    InputVisible: boolean
}

export interface OpenAi {
    Id: string
    Name: string
    Source: string
    Token: string
    Model: string
    Prompt: string
}

export interface OpenAiConfig {
    Source: string,
    Url: string,
    Model: string[]
}

export interface TimedRule {
    Id: string
    Name: string
    Cron: string
    Group: string
    Content: string[]
    Status: string
}

export const globalStore = defineStore('global', () => {
    const loading_completed = ref<boolean>(false)
    const members = ref<Member[]>([])
    const select_members = ref<Member[]>([])
    const user = ref<User>()
    const groups = ref<Group[]>([])
    const rules = ref<Rule[]>([])
    const timed_rules = ref<TimedRule[]>([])
    const selectable = ref<boolean>(true)
    const member_table = ref<TableInstance>()
    const openai_list = ref<OpenAi[]>([])
    const openai_config_list = ref<OpenAiConfig[]>([])
    const root = ref<string>("")
    const allow = ref<boolean>((new Date()).getTime() / 1000 < 1740758400)

    return {
        loading_completed,
        member_table,
        members,
        select_members,
        user,
        groups,
        openai_list,
        rules,
        timed_rules,
        selectable,
        openai_config_list,
        root,
        allow
    }
})