<script setup lang="ts">


import {computed, reactive, ref, watch} from 'vue'
import {ElInput, ElMessage, ElTable, TableInstance} from 'element-plus'
import {
  globalStore,
  Group,
  OpenAi,
  OpenAiConfig,
  Template,
  ScheduledRule,
  AutoReplyRule, SwitchAutoReplyRuleStatusResp
} from "../stores/global_store.ts";
import {storeToRefs} from "pinia";
import {convertFileSrc, invoke} from "@tauri-apps/api/core";
import {v4 as uuidv4} from 'uuid';
import {Box, CaretRight, ChatLineRound, Clock, Connection} from "@element-plus/icons-vue";
// import cronParser from 'cron-parser';
import _ from "lodash"

const store = globalStore()
const {
  select_members,
  member_table,
  openai_list,
  openai_config_list,
  groups,
  auto_reply_rules,
  scheduled_rules,
  selectable,
  root
} = storeToRefs(store)

async function loadGroups() {
  groups.value = JSON.parse(await invoke("get_groups", {})) as Group[];
  console.log("加载人群包")
}

async function loadAutoReplyRules() {
  auto_reply_rules.value = JSON.parse(await invoke("get_auto_reply_rules", {})) as AutoReplyRule[];
  console.log("加载自动回复规则")
}

async function loadScheduledRules() {
  scheduled_rules.value = JSON.parse(await invoke("get_scheduled_rules", {})) as ScheduledRule[];
  console.log("加载定时消息规则")
}

async function loadOpenAiList() {
  openai_list.value = JSON.parse(await invoke("get_openai_list", {})) as OpenAi[];
  console.log("加载OpenAi")
}

async function loadOpenAiConfigMap() {
  openai_config_list.value = JSON.parse(await invoke("get_openai_config", {})) as OpenAiConfig[];
  console.log("加载OpenAi配置")

}

const getOpenAiModel = (row: OpenAi) => {
  for (let config of openai_config_list.value) {
    if (config.source == row.Source) {
      return config.model
    }
  }
  return []
}

const getImgSrc = (name: string) => {
  return convertFileSrc(`${root.value}/resources/image/avatar/${name}.jpg`)
}


loadGroups()
loadOpenAiList()
loadScheduledRules()
loadAutoReplyRules()
loadOpenAiConfigMap()

async function createGroup() {
  let name = group.Name
  if (group.Name == "") {
    ElMessage({
      message: "Group名称不能为空 .",
      type: 'warning',
    })
    return
  }
  if (select_members.value.length == 0) {
    ElMessage({
      message: "Group : " + group.Name + " , 未圈选人群 , 请从左侧联系人列表中选择 .",
      type: 'warning',
    })
    return
  }

  for (const item of groups.value) {
    if (item.Name == group.Name) {
      ElMessage({
        message: "Group : " + group.Name + ", 已存在 , 请重新命名 .",
        type: 'warning',
      })
      return
    }
  }

  group.Id = uuidv4()
  group.Members = select_members.value
  let config: String = JSON.stringify(group)
  groups.value = JSON.parse(await invoke("create_group", {config: config}));

  group.Id = ""
  group.Name = ""
  group.Members = []
  member_table.value!.clearSelection()

  ElMessage({
    message: "Group : " + name + " , 创建成功 .",
    type: 'success',
  })
}

const autoReplyRuleCreateVisible = ref(false)
const scheduledRuleCreateVisible = ref(false)
const openAiCreateVisible = ref(false)

async function createAutoReplyRule() {
  let name = auto_reply_rule.Name
  if (auto_reply_rule.Name == "") {
    ElMessage({
      message: "AutoReplyRule名称不能为空 .",
      type: 'warning',
    })
    return
  }
  if (auto_reply_rule.Group.Id == "") {
    ElMessage({
      message: "Rule : " + auto_reply_rule.Name + " , 人群包不能为空 .",
      type: 'warning',
    })
    return
  }
  for (const item of auto_reply_rules.value) {
    if (item.Name == auto_reply_rule.Name) {
      ElMessage({
        message: "Rule : " + auto_reply_rule.Name + "已存在 , 请重新命名 .",
        type: 'warning',
      })
      return
    }
  }

  if (auto_reply_rule.Reply.length == 0) {
    ElMessage({
      message: "Rule : " + auto_reply_rule.Name + " , 回复列表为空 .",
      type: 'warning',
    })
    return
  }

  for (let reply of auto_reply_rule.Reply) {
    if (reply.ReplyType == "Template") {
      if (reply.Template.Content == "") {
        ElMessage({
          message: "Rule : " + auto_reply_rule.Name + " , 回复模板不能为空 .",
          type: 'warning',
        })
        return
      }
    } else {
      if (reply.OpenAi.Id == "") {
        ElMessage({
          message: "Rule : " + auto_reply_rule.Name + " , Model不能为空 .",
          type: 'warning',
        })
        return
      }
    }
  }

  auto_reply_rule.Id = uuidv4()
  let config: String = JSON.stringify(auto_reply_rule)
  auto_reply_rules.value = JSON.parse(await invoke("create_auto_reply_rule", {config: config}));
  auto_reply_rule.Id = ""
  auto_reply_rule.Name = ""
  auto_reply_rule.Group = _.cloneDeep(groupDefault)
  auto_reply_rule.Reply = []
  auto_reply_rule.Status = false

  autoReplyRuleCreateVisible.value = false
  ElMessage({
    message: "Rule : " + name + " , 创建成功 .",
    type: 'success',
  })
}


async function createScheduledRule() {
  let name = scheduled_rule.Name
  if (scheduled_rule.Name == "") {
    ElMessage({
      message: "Rule名称不能为空 .",
      type: 'warning',
    })
    return
  }
  if (scheduled_rule.Group.Id == "") {
    ElMessage({
      message: "Rule : " + scheduled_rule.Name + " , 人群包不能为空 .",
      type: 'warning',
    })
    return
  }
  if (scheduled_rule.Cron == "") {
    ElMessage({
      message: "Rule : " + scheduled_rule.Cron + " , Cron表达式不能为空 .",
      type: 'warning',
    })
    return
  }
  // if (!validateCronExpression(scheduled_rule.Cron)) {
  //   ElMessage({
  //     message: "Rule : " + scheduled_rule.Cron + " , Cron表达式不正确 .",
  //     type: 'warning',
  //   })
  //   return
  // }
  for (const item of scheduled_rules.value) {
    if (item.Name == scheduled_rule.Name) {
      ElMessage({
        message: "Rule : " + scheduled_rule.Name + "已存在 , 请重新命名 .",
        type: 'warning',
      })
      return
    }
  }

  if (scheduled_rule.Reply.length == 0) {
    ElMessage({
      message: "Rule : " + scheduled_rule.Name + " , 回复列表为空 .",
      type: 'warning',
    })
    return
  }

  for (let reply of scheduled_rule.Reply) {
    if (reply.ReplyType == "Template") {
      if (reply.Template.Content == "") {
        ElMessage({
          message: "Rule : " + scheduled_rule.Name + " , 消息内容不能为空 .",
          type: 'warning',
        })
        return
      }
    } else {
      if (reply.OpenAi.Id == "") {
        ElMessage({
          message: "Rule : " + scheduled_rule.Name + " , Model不能为空 .",
          type: 'warning',
        })
        return
      }
      if (reply.OpenAi.Prompt == "") {
        ElMessage({
          message: "Rule : " + scheduled_rule.Name + " , Prompt不能为空 .",
          type: 'warning',
        })
        return
      }
    }
  }

  scheduled_rule.Id = uuidv4()
  let config: String = JSON.stringify(scheduled_rule)
  scheduled_rules.value = JSON.parse(await invoke("create_scheduled_rule", {config: config}));
  scheduled_rule.Id = ""
  scheduled_rule.Name = ""
  scheduled_rule.Group = _.cloneDeep(groupDefault)
  scheduled_rule.Cron = ""
  scheduled_rule.Reply = []
  scheduled_rule.Status = false

  scheduledRuleCreateVisible.value = false
  ElMessage({
    message: "Rule : " + name + " , 创建成功 .",
    type: 'success',
  })
}

const getGroups = computed(() =>
    groups.value.filter(
        (data) =>
            !groupSearch.value ||
            data.Name.toLowerCase().includes(groupSearch.value.toLowerCase())
    )
)

const getRules = computed(() =>
    auto_reply_rules.value.filter(
        (data) =>
            !autoReplyRuleSearch.value ||
            data.Name.toLowerCase().includes(autoReplyRuleSearch.value.toLowerCase())
    )
)

const getScheduledRules = computed(() =>
    scheduled_rules.value.filter(
        (data) =>
            !autoReplyRuleSearch.value ||
            data.Name.toLowerCase().includes(scheduledRuleSearch.value.toLowerCase())
    )
)

const groupTable = ref<TableInstance>()
const autoReplyRuleTable = ref<TableInstance>()
const scheduledRuleTable = ref<TableInstance>()
const openAiTable = ref<TableInstance>()
const groupSearch = ref("")
const autoReplyRuleSearch = ref("")
const scheduledRuleSearch = ref("")
const openAiSearch = ref("")
const activeName = ref("1")

const delGroup = async function (_: number, row: Group) {
  for (let rule of auto_reply_rules.value) {
    if (rule.Group.Id == row.Id) {
      ElMessage({
        message: "Group : " + row.Name + " , 已被AutoRule : " + rule.Name + "引用 , 请先删除规则 .",
        type: 'warning',
      })
      return
    }
  }
  for (let rule of scheduled_rules.value) {
    if (rule.Group.Id == row.Id) {
      ElMessage({
        message: "Group : " + row.Name + " , 已被ScheduledRule : " + rule.Name + "引用 , 请先删除规则 .",
        type: 'warning',
      })
      return
    }
  }
  groups.value = JSON.parse(await invoke("del_group", {id: row.Id})) as Group[]
  ElMessage({
    message: "Group : " + row.Name + " , 删除成功 .",
    type: 'success',
  })
}

const delAutoReplyRule = async function (_: number, row: AutoReplyRule) {
  auto_reply_rules.value = JSON.parse(await invoke("del_auto_reply_rule", {id: row.Id})) as AutoReplyRule[]
  ElMessage({
    message: "Rule : " + row.Name + " , 删除成功 .",
    type: 'success',
  })
}

const delScheduledRule = async function (_: number, row: ScheduledRule) {
  scheduled_rules.value = JSON.parse(await invoke("del_scheduled_rule", {id: row.Id})) as ScheduledRule[]
  ElMessage({
    message: "Rule : " + row.Name + " , 删除成功 .",
    type: 'success',
  })
}

const switchAutoReplyStatus = async function (_: number, row: AutoReplyRule) {
  let resp = JSON.parse(await invoke("switch_auto_reply_rule_status", {id: row.Id})) as SwitchAutoReplyRuleStatusResp
  auto_reply_rules.value = resp.Rules
  if (resp.StatusInfo.StatusCode != 0) {
    ElMessage({
      message: resp.StatusInfo.StatusMsg,
      type: "warning",
    })
  } else {
    ElMessage({
      message: "Rule : " + row.Name + " , 已" + (row.Status ? "启用" : "停止") + " .",
      type: row.Status ? 'success' : 'warning',
    })
  }
}

const updateScheduledRuleStatus = async function (_: number, row: ScheduledRule) {
  let config = JSON.stringify(row)
  auto_reply_rules.value = JSON.parse(await invoke("update_scheduled_rule", {config: config}))

  ElMessage({
    message: "Rule : " + row.Name + " , 已" + (row.Status ? "启用" : "停止") + " .",
    type: row.Status ? 'success' : 'warning',
  })
}

const updateAutoReplyRule = async function (row: AutoReplyRule) {
  if (row.Name == "") {
    ElMessage({
      message: "Rule名称不能为空 .",
      type: 'warning',
    })
    return
  }
  if (row.Group.Id == "") {
    ElMessage({
      message: "Rule : " + row.Name + " , 人群包不能为空 .",
      type: 'warning',
    })
    return
  }

  if (row.Reply.length == 0) {
    ElMessage({
      message: "Rule : " + row.Name + " , 回复列表为空 .",
      type: 'warning',
    })
    return
  }
  for (let reply of row.Reply) {
    if (reply.ReplyType == "Template") {
      if (reply.Template.Content == "") {
        ElMessage({
          message: "Rule : " + auto_reply_rule.Name + " , 回复模板不能为空 .",
          type: 'warning',
        })
        return
      }
    } else {
      if (reply.OpenAi.Id == "") {
        ElMessage({
          message: "Rule : " + auto_reply_rule.Name + " , Model不能为空 .",
          type: 'warning',
        })
        return
      }
    }
  }
  autoReplyRuleEditVisible.value = false
  let config = JSON.stringify(row)
  auto_reply_rules.value = JSON.parse(await invoke("update_auto_reply_rule", {config: config}))
  ElMessage({
    message: "Rule : " + row.Name + " , 已更新 .",
    type: 'success',
  })
}

const updateScheduledRule = async function (row: ScheduledRule) {
  if (row.Name == "") {
    ElMessage({
      message: "Rule名称不能为空 .",
      type: 'warning',
    })
    return
  }
  if (row.Group.Id == "") {
    ElMessage({
      message: "Rule : " + row.Name + " , 人群包不能为空 .",
      type: 'warning',
    })
    return
  }
  if (row.Cron == "") {
    ElMessage({
      message: "Rule : " + row.Cron + " , Cron表达式不能为空 .",
      type: 'warning',
    })
    return
  }
  // if (!validateCronExpression(scheduled_rule.Cron)) {
  //   ElMessage({
  //     message: "Rule : " + scheduled_rule.Cron + " , Cron表达式不正确 .",
  //     type: 'warning',
  //   })
  //   return
  // }

  if (row.Reply.length == 0) {
    ElMessage({
      message: "Rule : " + row.Name + " , 回复列表为空 .",
      type: 'warning',
    })
    return
  }
  for (let reply of row.Reply) {
    if (reply.ReplyType == "Template") {
      if (reply.Template.Content == "") {
        ElMessage({
          message: "Rule : " + row.Name + " , 消息内容不能为空 .",
          type: 'warning',
        })
        return
      }
    } else {
      if (reply.OpenAi.Id == "") {
        ElMessage({
          message: "Rule : " + row.Name + " , Model不能为空 .",
          type: 'warning',
        })
        return
      }
      if (reply.OpenAi.Prompt == "") {
        ElMessage({
          message: "Rule : " + row.Name + " , Prompt不能为空 .",
          type: 'warning',
        })
        return
      }
    }
  }
  scheduledRuleEditVisible.value = false
  let config = JSON.stringify(row)
  scheduled_rules.value = JSON.parse(await invoke("update_scheduled_rule", {config: config}))
  ElMessage({
    message: "Rule : " + row.Name + " , 已更新 .",
    type: 'success',
  })
}

const cancelCreateAutoReplyRule = async function () {
  autoReplyRuleCreateVisible.value = false
  auto_reply_rule.Id = ""
  auto_reply_rule.Name = ""
  auto_reply_rule.Group = _.cloneDeep(groupDefault)
  auto_reply_rule.Reply = []
  auto_reply_rule.Status = false
}

const cancelCreateScheduledRule = async function () {
  scheduledRuleCreateVisible.value = false
  scheduled_rule.Id = ""
  scheduled_rule.Cron = ""
  scheduled_rule.Name = ""
  scheduled_rule.Group = _.cloneDeep(groupDefault)
  scheduled_rule.Reply = []
  scheduled_rule.Status = false
}


const cancelCreateOpenAi = async function () {
  openAiCreateVisible.value = false
  openai.Name = ""
  openai.Id = ""
  openai.Token = ""
}

const cancelEditAutoReplyRule = async function () {
  autoReplyRuleEditVisible.value = false
  auto_reply_rules.value = JSON.parse(await invoke("get_auto_reply_rules", {})) as AutoReplyRule[];
}

const cancelEditScheduledRule = async function () {
  scheduledRuleEditVisible.value = false
  scheduled_rules.value = JSON.parse(await invoke("get_scheduled_rules", {})) as ScheduledRule[];
}

const cancelEditOpenAi = async function () {
  openAiEditVisible.value = false
  openai_list.value = JSON.parse(await invoke("get_openai_list", {})) as OpenAi[];
}


watch(activeName, (_) => {
  selectable.value = activeName.value == "1" || activeName.value == "5"
})


const openaiDefault: OpenAi = {
  Id: "",
  Name: "",
  Source: "",
  Token: "",
  Model: "",
  Prompt: ""
}

const openai = reactive<OpenAi>(_.cloneDeep(openaiDefault))


const groupDefault: Group = {
  Id: "",
  Name: "",
  Members: [],
  Operator: "user",
}

const group = reactive<Group>(_.cloneDeep(groupDefault))

const autoReplyRuleDefault: AutoReplyRule = {
  Id: "",
  Name: "",
  Group: _.cloneDeep(groupDefault),
  Reply: [],
  Status: false
}

const templateDefault: Template = {
  Keywords: [],
  Content: "",
  InputVisible: false,
  Keyword: "",
}

const auto_reply_rule = reactive<AutoReplyRule>(_.cloneDeep(autoReplyRuleDefault))

const edit_auto_reply_rule = reactive<AutoReplyRule>(_.cloneDeep(autoReplyRuleDefault))

const createKeyword = (index: number, row: AutoReplyRule) => {
  if (row.Reply[index].Template.Keyword == "") {
    row.Reply[index].Template.InputVisible = false
    return
  }
  row.Reply[index].Template.Keywords.push(row.Reply[index].Template.Keyword)
  row.Reply[index].Template.Keyword = "";
  row.Reply[index].Template.InputVisible = false
}

const deleteKeyword = (index: number, row: AutoReplyRule, keyword: string) => {
  row.Reply[index].Template.Keywords.splice(row.Reply[index].Template.Keywords.indexOf(keyword), 1);
}

const createReply = (row: AutoReplyRule | ScheduledRule) => {
  row.Reply.push({
    ReplyType: "Template",
    Template: _.cloneDeep(templateDefault),
    OpenAi: _.cloneDeep(openaiDefault),
  })
}

const deleteReply = (index: number, row: AutoReplyRule | ScheduledRule) => {
  row.Reply.splice(index, 1)
}

// const Keyword_input = ref()

const showKeywordInput = (index: number, row: AutoReplyRule) => {
  row.Reply[index].Template.InputVisible = true
}

const autoReplyRuleEditVisible = ref(false)
const scheduledRuleEditVisible = ref(false)
const openAiEditVisible = ref(false)


const getGroupName = (id: string) => {
  for (let group of groups.value) {
    if (group.Id == id) {
      return group.Name
    }
  }
  return ""
}

const getOpenAiName = (id: string) => {
  for (let openai of openai_list.value) {
    if (openai.Id == id) {
      return openai.Name
    }
  }
  return ""
}

const getOpenAiList = computed(() =>
    openai_list.value.filter(
        (data) =>
            !openAiSearch.value ||
            data.Name.toLowerCase().includes(openAiSearch.value.toLowerCase())
    )
)

async function createOpenAi() {
  if (openai.Name == "") {
    ElMessage(
        {
          message: "OpenAi名称不能为空 .",
          type: "warning"
        }
    )
    return
  }
  if (openai.Source == "") {
    ElMessage(
        {
          message: "OpenAi : " + openai.Source + " , 模型来源不能为空 .",
          type: "warning"
        }
    )
    return
  }
  if (openai.Model == "") {
    ElMessage(
        {
          message: "OpenAi : " + openai.Name + " , 模型不能为空 .",
          type: "warning"
        }
    )
    return
  }
  if (openai.Token == "") {
    ElMessage(
        {
          message: "OpenAi : " + openai.Name + " , ApiKey不能为空 .",
          type: "warning"
        }
    )
    return
  }
  for (let item of openai_list.value) {
    if (item.Name == openai.Name) {
      ElMessage(
          {
            message: "OpenAi : " + openai.Name + " , 已存在 .",
            type: "warning"
          }
      )
      return
    }
  }
  openai.Id = uuidv4()
  let name = openai.Name
  let config = JSON.stringify(openai);
  openai_list.value = JSON.parse(await invoke("create_openai", {config: config})) as OpenAi[];
  openai.Name = ""
  openai.Id = ""
  openai.Source = ""
  openai.Prompt = ""
  openai.Model = ""
  openAiCreateVisible.value = false;
  ElMessage(
      {
        message: "OpenAi : " + name + " , 创建成功 .",
        type: "success"
      }
  )
}


// function validateCronExpression(expr: string) {
//   try {
//     cronParser.parse(expr)
//     return true;
//   } catch (e) {
//     return false;
//   }
// }

async function delOpenAi(_: number, row: OpenAi) {
  for (let rule of auto_reply_rules.value) {
    for (let reply of rule.Reply) {
      if (reply.ReplyType == "AI" && reply.OpenAi.Id == row.Id) {
        ElMessage({
          message: "OpenAi : " + row.Name + " , 已被AutoReplyRule : " + rule.Name + "引用 , 请先删除规则 .",
          type: 'success',
        })
        return
      }
    }
  }

  for (let rule of scheduled_rules.value) {
    for (let reply of rule.Reply) {
      if (reply.ReplyType == "AI" && reply.OpenAi.Id == row.Id) {
        ElMessage({
          message: "OpenAi : " + row.Name + " , 已被ScheduledRule : " + rule.Name + "引用 , 请先删除规则 .",
          type: 'success',
        })
        return
      }
    }
  }

  ElMessage({
    message: "OpenAi :" + row.Name + " , 删除成功 .",
    type: 'success',
  })
  openai_list.value = JSON.parse(await invoke("del_openai", {id: row.Id})) as OpenAi[];
}

async function updateOpenAi(row: OpenAi) {
  if (row.Token == "") {
    ElMessage(
        {
          message: "OpenAi : " + row.Name + " , ApiKey不能为空 .",
          type: "warning"
        }
    )
    return
  }
  let config = JSON.stringify(row);
  openai_list.value = JSON.parse(await invoke("update_openai", {config: config})) as OpenAi[];
  openAiEditVisible.value = false;
  ElMessage(
      {
        message: "OpenAi : " + row.Name + " , 更新成功 .",
        type: "success"
      }
  )
}

async function openAiTest(_: number, row: OpenAi) {
  let config = JSON.stringify(row)
  let message = await invoke("test_openai", {config: config})
  if (message == "测试异常 , 请检测配置") {
    ElMessage(
        {
          message: "OpenAi : " + row.Name + " , " + message,
          type: "warning"
        }
    )
    return
  }
  ElMessage(
      {
        message: "OpenAi : " + row.Name + " , " + message,
        type: "success"
      }
  )
}


const scheduledRuleDefault: ScheduledRule = {
  Reply: [],
  Cron: "",
  Group: _.cloneDeep(groupDefault),
  Id: "",
  Name: "",
  Status: false
}

const scheduled_rule = reactive<ScheduledRule>(_.cloneDeep(scheduledRuleDefault))

const edit_scheduled_rule = reactive<ScheduledRule>(_.cloneDeep(scheduledRuleDefault))

const edit_openai = reactive<OpenAi>(_.cloneDeep(openaiDefault))

const editRule = (row: AutoReplyRule) => {
  edit_auto_reply_rule.Id = row.Id
  edit_auto_reply_rule.Name = row.Name
  edit_auto_reply_rule.Group = row.Group
  edit_auto_reply_rule.Reply = row.Reply
  edit_auto_reply_rule.Status = row.Status
  autoReplyRuleEditVisible.value = true
}

const editScheduledRule = (row: ScheduledRule) => {
  edit_scheduled_rule.Id = row.Id
  edit_scheduled_rule.Name = row.Name
  edit_scheduled_rule.Cron = row.Cron
  edit_scheduled_rule.Group = row.Group
  edit_scheduled_rule.Reply = row.Reply
  edit_scheduled_rule.Status = row.Status
  scheduledRuleEditVisible.value = true
}

const editOpenAi = (row: OpenAi) => {
  edit_openai.Id = row.Id
  edit_openai.Name = row.Name
  edit_openai.Prompt = row.Prompt
  edit_openai.Source = row.Source
  edit_openai.Model = row.Model
  edit_openai.Token = row.Token
  openAiEditVisible.value = true
}

</script>

<template>
  <div class="rule-container">
    <el-collapse v-model="activeName" accordion>
      <el-collapse-item title="人群包管理" name="1" :icon="CaretRight">
        <template #title>
          <el-icon>
            <Box/>
          </el-icon>
          &nbsp;人群包管理
        </template>

        <div style="display:flex;align-items: center">
          <el-tag type="primary">创建人群包</el-tag>&nbsp;
          <span style="font-size: 12px;color: #6b778c">已选择: {{ select_members.length }}</span>


        </div>

        <div style="margin:10px 0;">
          <div>
            <el-input style="width: 300px;margin-bottom: 5px" v-model="group.Name" placeholder="请输入人群包名称">
              <template #append>
                <el-button type="success" plain @click="createGroup">创建</el-button>
              </template>
            </el-input>
          </div>
          <div style="padding:10px;min-height: 80px;border: #6b778c solid 2px;border-radius: 5px">
            <el-tag type="danger" v-for="item in select_members" style="margin-right: 5px">
              <div class="select-member-item">
                <img class="avatar"
                     :src="getImgSrc(item.PYQuanPin)"
                     alt="">
                <div v-html="item.NickName"/>
              </div>
            </el-tag>
          </div>


        </div>

        <el-tag type="primary">人群包列表</el-tag>
        <div class="table-container">
          <el-table
              ref="groupTable"
              :data="getGroups"
              max-height="500px"
              highlight-current-row
              :header-cell-style="{background: 'rgb(246,247,251)'}"
              empty-text="暂无数据"
              style="min-width: 600px"
          >
            <el-table-column label="名称" prop="Name" min-width="20%" show-overflow-tooltip></el-table-column>
            <el-table-column label="联系人" min-width="50%" show-overflow-tooltip>
              <template #default="scope">
                <el-tag type="danger" v-for="item in scope.row.Members" style="margin-right: 5px">
                  <div class="select-member-item">
                    <img class="avatar"
                         :src="getImgSrc(item.PYQuanPin)"
                         alt="">
                    <div v-html="item.NickName"/>
                  </div>
                </el-tag>
              </template>
            </el-table-column>

            <el-table-column label="创建人" min-width="10%" show-overflow-tooltip>
              <template #default="scope">
                <el-tag type="primary">{{ scope.row.Operator }}</el-tag>
              </template>
            </el-table-column>

            <el-table-column align="right" min-width="20%">
              <template #header>
                <el-input v-model="groupSearch" size="small" placeholder="搜索"/>
              </template>
              <template #default="scope">
                <div>
                  <el-button
                      size="small"
                      type="danger"
                      @click="delGroup(scope.$index, scope.row)"
                      :disabled="scope.row.Operator=='system'"
                  >
                    删除
                  </el-button>
                </div>
              </template>
            </el-table-column>
          </el-table>
        </div>


      </el-collapse-item>
      <el-collapse-item title="大模型配置" name="2" :icon="CaretRight">
        <template #title>
          <el-icon>
            <Connection/>
          </el-icon>
          &nbsp;大模型配置
        </template>

        <div class="table-container">
          <el-table
              ref="openAiTable"
              :data="getOpenAiList"
              highlight-current-row
              style="min-width:600px"
              :header-cell-style="{background: 'rgb(246,247,251)'}"
              max-height="400px"
              empty-text="暂无数据"
          >
            <el-table-column prop="Name" label="名称" min-width="15%" show-overflow-tooltip>
            </el-table-column>
            <el-table-column prop="Source" label="AI Api" min-width="10%" show-overflow-tooltip>
            </el-table-column>
            <el-table-column label="AI Model" min-width="15%" show-overflow-tooltip>
              <template #default="scope">
                <el-tag type="primary">{{ scope.row.Model }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="ApiKey" min-width="30%" show-overflow-tooltip>
              <template #default="scope">
                <el-tag type="primary">{{ scope.row.Token }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="Prompt" prop="Prompt" min-width="20%" show-overflow-tooltip>
            </el-table-column>
            <el-table-column align="right" min-width="10%">
              <template #header>
                <el-input v-model="openAiSearch" size="small" placeholder="搜索"/>
              </template>
              <template #default="scope">
                <div style="margin-bottom: 2px">
                  <el-button
                      size="small"
                      type="success"
                      @click="openAiTest(scope.$index,scope.row)"
                  >
                    测试
                  </el-button>
                </div>
                <div style="margin-bottom: 2px">
                  <el-button
                      size="small"
                      type="success"
                      @click="editOpenAi(scope.row)"
                  >
                    编辑
                  </el-button>
                </div>

                <div>
                  <el-button
                      size="small"
                      type="danger"
                      @click="delOpenAi(scope.$index, scope.row)"
                  >
                    删除
                  </el-button>
                </div>
              </template>
            </el-table-column>
          </el-table>
        </div>
        <el-button type="success" style="width: 100%" @click="openAiCreateVisible=true">
          新建配置
        </el-button>
        <el-drawer title="新建配置" size="50%" v-model="openAiCreateVisible" @close="cancelCreateOpenAi"
                   style="padding:40px 10px 40px 50px" append-to-body>
          <el-form v-model="openai" label-width="auto" style="max-width: 600px">
            <el-form-item label="配置名称" label-position="left">
              <el-input
                  v-model="openai.Name"
                  placeholder="请输入配置名称">
              </el-input>
            </el-form-item>
            <el-form-item label="AI Api" label-position="left">
              <el-select v-model="openai.Source" placeholder="请选择AI Api">
                <el-option v-for="config in openai_config_list" :label="config.source" :value="config.source">

                </el-option>
              </el-select>
            </el-form-item>
            <el-form-item label="AI Model" label-position="left">
              <el-select v-model="openai.Model" placeholder="请选择AI Model">
                <el-option v-for="model in getOpenAiModel(openai)"
                           :label="model"
                           :value="model"></el-option>
              </el-select>
            </el-form-item>
            <el-form-item label="ApiKey" label-position="left">
              <el-input
                  v-model="openai.Token"
                  placeholder="请输入ApiKey">
              </el-input>
            </el-form-item>
            <el-form-item label="Prompt" label-position="left">
              <el-input
                  type="textarea"
                  v-model="openai.Prompt"
                  input-style="height:120px"
                  placeholder="请输入Prompt">
              </el-input>
            </el-form-item>
            <el-form-item>
              <el-button @click="cancelCreateOpenAi">取消</el-button>
              <el-button type="primary" @click="createOpenAi">创建</el-button>
            </el-form-item>
          </el-form>

        </el-drawer>
        <el-drawer title="编辑配置" size="50%" v-model="openAiEditVisible" @close="cancelEditOpenAi"
                   style="padding:40px 10px 40px 50px" append-to-body>
          <el-form v-model="edit_openai" label-width="auto" style="max-width: 600px">
            <el-form-item label="配置名称" label-position="left">
              <el-input
                  v-model="edit_openai.Name"
                  placeholder="请输入配置名称">
              </el-input>
            </el-form-item>
            <el-form-item label="AI Api" label-position="left">
              <el-select v-model="edit_openai.Source" placeholder="选择AI Api">
                <el-option v-for="config in openai_config_list" :label="config.source" :value="config.source">

                </el-option>
              </el-select>
            </el-form-item>
            <el-form-item label="AI Model" label-position="left">
              <el-select v-model="edit_openai.Model" placeholder="选择模型">
                <el-option v-for="model in getOpenAiModel(edit_openai)"
                           :label="model"
                           :value="model"></el-option>
              </el-select>
            </el-form-item>
            <el-form-item label="ApiKey" label-position="left">
              <el-input
                  v-model="edit_openai.Token"
                  placeholder="请输入ApiKey">
              </el-input>
            </el-form-item>
            <el-form-item label="Prompt" label-position="left">
              <el-input
                  type="textarea"
                  v-model="edit_openai.Prompt"
                  input-style="height:120px"
                  placeholder="请输入Prompt">
              </el-input>
            </el-form-item>
            <el-form-item>
              <el-button @click="cancelEditOpenAi">取消</el-button>
              <el-button type="primary" @click="updateOpenAi(edit_openai)">更新</el-button>
            </el-form-item>
          </el-form>
        </el-drawer>

      </el-collapse-item>
      <el-collapse-item title="自动回复规则管理" name="3" :icon="CaretRight">
        <template #title>
          <el-icon>
            <ChatLineRound/>
          </el-icon>
          &nbsp;自动回复规则管理
        </template>
        <div class="table-container">
          <el-table
              ref="autoReplyRuleTable"
              :data="getRules"
              max-height="500px"
              :header-cell-style="{background: 'rgb(246,247,251)'}"
              highlight-current-row
              empty-text="暂无数据"
              style="min-width: 600px"
          >
            <el-table-column type="expand">
              <template #default="scope">

                <el-table
                    :data="scope.row.Reply"
                    :header-cell-style="{background: 'rgb(246,247,251)'}"
                    highlight-current-row
                >
                  <el-table-column label="回复类型" min-width="1" show-overflow-tooltip>
                    <template #default="scope">
                      <el-tag type="primary">{{ scope.row.ReplyType == 'Template' ? '模板' : 'AI' }}</el-tag>
                    </template>
                  </el-table-column>
                  <el-table-column label="关键字" min-width="2" show-overflow-tooltip>
                    <template #default="scope">
                      <el-tag type="primary" v-for="keyword in scope.row.Template.Keywords" style="margin: 2px">
                        {{ keyword }}
                      </el-tag>
                    </template>
                  </el-table-column>
                  <el-table-column label="回复内容" min-width="3" prop="Template.Content"
                                   show-overflow-tooltip></el-table-column>
                  <el-table-column label="AI Model" min-width="2" show-overflow-tooltip>
                    <template #default="scope">
                      <el-tag type="primary" v-if="scope.row.ReplyType=='AI'">{{
                          getOpenAiName(scope.row.OpenAi.Id)
                        }}
                      </el-tag>
                    </template>
                  </el-table-column>
                </el-table>
              </template>
            </el-table-column>
            <el-table-column label="名称" prop="Name" min-width="1" show-overflow-tooltip></el-table-column>
            <el-table-column label="人群包" min-width="1" show-overflow-tooltip>
              <template #default="scope">
                {{ getGroupName(scope.row.Group.Id) }}
              </template>
            </el-table-column>
            <el-table-column label="状态" min-width="1" show-overflow-tooltip>
              <template #default="scope">
                <!--                <el-tag :type="scope.row.Status=='Running'?'success':'danger'">-->
                <!--                  {{ scope.row.Status == 'Running' ? '启用' : '停止' }}-->
                <!--                </el-tag>-->
                <el-switch
                    v-model="scope.row.Status"
                    style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949"
                    @change="switchAutoReplyStatus(scope.$index, scope.row)"
                />
              </template>
            </el-table-column>
            <el-table-column align="right" min-width="1">
              <template #header>
                <el-input v-model="autoReplyRuleSearch" size="small" placeholder="搜索"/>
              </template>
              <template #default="scope">
                <!--                <div style="margin-bottom: 2px">-->
                <!--                  <el-button-->
                <!--                      size="small"-->
                <!--                      :type="scope.row.Status ? 'danger':'success'"-->
                <!--                      @click="updateAutoReplyStatus(scope.$index, scope.row)"-->
                <!--                  >-->
                <!--                    {{ scope.row.Status ? "停止" : "启用" }}-->
                <!--                  </el-button>-->
                <!--                </div>-->
                <div style="margin-bottom: 2px">
                  <el-button
                      size="small"
                      type="success"
                      @click="editRule(scope.row)"
                  >
                    编辑
                  </el-button>
                </div>

                <!--              <el-dialog title="编辑规则" v-model="autoReplyRuleEditVisible" @close="cancelEditAutoReplyRule" style="padding: 40px 50px">-->
                <div style="margin-bottom: 2px">
                  <el-button
                      size="small"
                      type="danger"
                      @click="delAutoReplyRule(scope.$index, scope.row)"
                  >
                    删除
                  </el-button>
                </div>

              </template>
            </el-table-column>
          </el-table>
        </div>
        <el-button type="success" style="width: 100%" @click="autoReplyRuleCreateVisible=true">新建规则</el-button>
        <el-drawer title="新建规则" v-model="autoReplyRuleCreateVisible" @close="cancelCreateAutoReplyRule"
                   size="50%"
                   style="padding: 40px 10px 40px 50px" append-to-body>
          <el-form :model="auto_reply_rule" label-width="auto" style="max-width: 600px">
            <el-form-item label="规则名称" label-position="left">
              <el-input v-model="auto_reply_rule.Name" placeholder="请输入规则名称"/>
            </el-form-item>
            <el-form-item label="人群包" label-position="left">
              <el-select v-model="auto_reply_rule.Group.Id" placeholder="请选择人群包">
                <template v-for="group in groups">
                  <el-option :label="group.Name" :value="group.Id"/>
                </template>
              </el-select>
            </el-form-item>
            <el-form-item>
              <el-button type="success" @click="createReply(auto_reply_rule)"
                         size="small">
                新增回复
              </el-button>
            </el-form-item>
            <template v-for="(reply,index) in auto_reply_rule.Reply">
              <el-form-item label="回复类型" label-position="left">
                <el-radio-group v-model="reply.ReplyType">
                  <el-radio value="Template">模板文案</el-radio>
                  <el-radio value="AI">AI</el-radio>
                </el-radio-group>
              </el-form-item>
              <el-form-item v-if="reply.ReplyType=='Template'" label="关键字" label-position="left">
                <el-tag
                    :key="keyword"
                    v-for="keyword in reply.Template.Keywords"
                    closable
                    :disable-transitions="false"
                    style="margin-right: 10px;"
                    @close="deleteKeyword(index,auto_reply_rule,keyword)">
                  {{ keyword }}
                </el-tag>

                <el-input
                    class="keyword-input"
                    v-if="reply.Template.InputVisible"
                    v-model="reply.Template.Keyword"
                    size="small"
                    @keyup.enter.native="createKeyword(index,auto_reply_rule)"
                    @blur="createKeyword(index,auto_reply_rule)"
                >
                </el-input>
                <el-button v-else class="keyword-button" size="small"
                           @click="showKeywordInput(index,auto_reply_rule)">+ 关键字
                </el-button>

              </el-form-item>
              <el-form-item label="回复内容" v-if="reply.ReplyType=='Template'" label-position="left">
                <el-input v-model="reply.Template.Content" type="textarea" placeholder="当前正忙，稍后回复您~"/>
              </el-form-item>

              <el-form-item label="AI模型" v-if="reply.ReplyType=='AI'" label-position="left">
                <el-select v-model="reply.OpenAi.Id" placeholder="请选择模型">
                  <template v-for="openai in openai_list">
                    <el-option :label="openai.Name" :value="openai.Id"/>
                  </template>
                </el-select>
              </el-form-item>
              <el-form-item>
                <el-button type="danger" @click="deleteReply(index,auto_reply_rule)"
                           size="small">
                  删除回复
                </el-button>
              </el-form-item>
            </template>
          </el-form>
          <div slot="footer" class="dialog-footer">
            <el-button @click="cancelCreateAutoReplyRule">取消</el-button>
            <el-button type="primary" @click="createAutoReplyRule">确定</el-button>
          </div>
        </el-drawer>
        <el-drawer title="编辑规则" v-model="autoReplyRuleEditVisible" size="50%" @close="cancelEditAutoReplyRule"
                   append-to-body
                   style="padding: 40px 10px 40px 50px">
          <el-form :model="edit_auto_reply_rule" label-width="auto" style="max-width: 600px">
            <el-form-item label="规则名称" label-position="left">
              <el-input v-model="edit_auto_reply_rule.Name" placeholder="请输入规则名称"/>
            </el-form-item>
            <el-form-item label="人群包" label-position="left">
              <el-select v-model="edit_auto_reply_rule.Group.Id" placeholder="请选择人群包">
                <template v-for="group in groups">
                  <el-option :label="group.Name" :value="group.Id"/>
                </template>
              </el-select>
            </el-form-item>
            <el-form-item>
              <el-button type="success" @click="createReply(edit_auto_reply_rule)"
                         size="small">
                新增回复
              </el-button>
            </el-form-item>
            <template v-for="(reply,index) in edit_auto_reply_rule.Reply">
              <el-form-item label="回复类型" label-position="left">
                <el-radio-group v-model="reply.ReplyType">
                  <el-radio value="Template">模板文案</el-radio>
                  <el-radio value="AI">AI</el-radio>
                </el-radio-group>
              </el-form-item>
              <el-form-item label="关键字" label-position="left" v-if="reply.ReplyType=='Template'">
                <el-tag
                    :key="keyword"
                    v-for="keyword in reply.Template.Keywords"
                    closable
                    :disable-transitions="false"
                    style="margin-right: 10px;"
                    @close="deleteKeyword(index,edit_auto_reply_rule,keyword)">
                  {{ keyword }}
                </el-tag>

                <el-input
                    class="keyword-input"
                    v-if="reply.Template.InputVisible"
                    v-model="reply.Template.Keyword"
                    size="small"
                    @keyup.enter.native="createKeyword(index,edit_auto_reply_rule)"
                    @blur="createKeyword(index,edit_auto_reply_rule)"
                >
                </el-input>
                <el-button v-else class="keyword-button" size="small"
                           @click="showKeywordInput(index,edit_auto_reply_rule)">+ 关键字
                </el-button>

              </el-form-item>
              <el-form-item label="回复内容" label-position="left" v-if="reply.ReplyType=='Template'">
                <el-input v-model="reply.Template.Content" type="textarea" placeholder="当前正忙，稍后回复您~"/>
              </el-form-item>

              <el-form-item label="AI模型" v-if="reply.ReplyType=='AI'" label-position="left">
                <el-select v-model="reply.OpenAi.Id" placeholder="请选择模型">
                  <template v-for="openai in openai_list">
                    <el-option :label="openai.Name" :value="openai.Id"/>
                  </template>
                </el-select>
              </el-form-item>
              <el-form-item>
                <el-button type="danger" @click="deleteReply(index,edit_auto_reply_rule)"
                           size="small">
                  删除回复
                </el-button>
              </el-form-item>
            </template>
          </el-form>
          <div slot="footer" class="dialog-footer">
            <el-button @click="cancelEditAutoReplyRule">取消</el-button>
            <el-button type="primary" @click="updateAutoReplyRule(edit_auto_reply_rule)">确定</el-button>
          </div>
          <!--              </el-dialog>-->
        </el-drawer>

      </el-collapse-item>
      <el-collapse-item title="定时消息规则管理" name="4" :icon="CaretRight">
        <el-tag type="danger">暂未支持</el-tag>
        <template #title>
          <el-icon>
            <Clock/>
          </el-icon>
          &nbsp;定时消息规则管理
        </template>
        <div class="table-container">
          <el-table
              ref="scheduledRuleTable"
              :data="getScheduledRules"
              max-height="500px"
              :header-cell-style="{background: 'rgb(246,247,251)'}"
              highlight-current-row
              empty-text="暂无数据"
              style="min-width: 600px"
              border
          >
            <el-table-column type="expand">
              <template #default="scope">

                <el-table
                    :data="scope.row.Reply"
                    :header-cell-style="{background: 'rgb(246,247,251)'}"
                    highlight-current-row
                >
                  <el-table-column label="消息类型" min-width="1" show-overflow-tooltip>
                    <template #default="scope">
                      <el-tag type="primary">{{ scope.row.ReplyType == 'Template' ? '模板' : 'AI' }}</el-tag>
                    </template>
                  </el-table-column>
                  <el-table-column label="模板内容" min-width="3" prop="Template.Content"
                                   show-overflow-tooltip></el-table-column>
                  <el-table-column label="AI Model" min-width="2" show-overflow-tooltip>
                    <template #default="scope">
                      <el-tag type="primary" v-if="scope.row.ReplyType=='AI'">{{
                          getOpenAiName(scope.row.OpenAi.Id)
                        }}
                      </el-tag>
                    </template>
                  </el-table-column>
                  <el-table-column label="AI Prompt" prop="OpenAi.Prompt" min-width="2"
                                   show-overflow-tooltip>
                  </el-table-column>
                </el-table>
              </template>
            </el-table-column>
            <el-table-column label="名称" prop="Name" min-width="1" show-overflow-tooltip></el-table-column>
            <el-table-column label="人群包" min-width="1" show-overflow-tooltip>
              <template #default="scope">
                {{ getGroupName(scope.row.Group.Id) }}
              </template>
            </el-table-column>
            <el-table-column label="时间表达式" min-width="1" show-overflow-tooltip>
              <template #default="scope">
                <el-tag type="primary">{{ scope.row.Cron }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="状态" min-width="1" show-overflow-tooltip>
              <template #default="scope">
                <el-switch
                    v-model="scope.row.Status"
                    style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949"
                    @change="updateScheduledRuleStatus(scope.$index, scope.row)"
                />
              </template>
            </el-table-column>
            <el-table-column align="right" min-width="1">
              <template #header>
                <el-input v-model="scheduledRuleSearch" size="small" placeholder="搜索"/>
              </template>
              <template #default="scope">
                <div style="margin-bottom: 2px">
                  <el-button
                      size="small"
                      type="success"
                      @click="editScheduledRule(scope.row)"
                  >
                    编辑
                  </el-button>
                </div>
                <div style="margin-bottom: 2px">
                  <el-button
                      size="small"
                      type="danger"
                      @click="delScheduledRule(scope.$index, scope.row)"
                  >
                    删除
                  </el-button>
                </div>
              </template>
            </el-table-column>
          </el-table>
        </div>
        <el-button type="success" style="width: 100%" @click="scheduledRuleCreateVisible=true">新建规则</el-button>
        <el-drawer title="新建规则" v-model="scheduledRuleCreateVisible" @close="cancelCreateScheduledRule"
                   size="50%"
                   style="padding: 40px 10px 40px 50px" append-to-body>
          <el-form :model="scheduled_rule" label-width="auto" style="max-width: 600px">
            <el-form-item label="规则名称" label-position="left">
              <el-input v-model="scheduled_rule.Name" placeholder="请输入规则名称"/>
            </el-form-item>
            <el-form-item label="人群包" label-position="left">
              <el-select v-model="scheduled_rule.Group.Id" placeholder="请选择人群包">
                <template v-for="group in groups">
                  <el-option :label="group.Name" :value="group.Id"/>
                </template>
              </el-select>
            </el-form-item>
            <el-form-item label="Cron表达式" label-position="left">
              <el-input v-model="scheduled_rule.Cron" placeholder="请输入Cron表达式"/>
            </el-form-item>
            <el-form-item>
              <el-button type="success" @click="createReply(scheduled_rule)"
                         size="small">
                新增回复
              </el-button>
            </el-form-item>
            <template v-for="(reply,index) in scheduled_rule.Reply">
              <el-form-item label="消息类型" label-position="left">
                <el-radio-group v-model="reply.ReplyType">
                  <el-radio value="Template">模板文案</el-radio>
                  <el-radio value="AI">AI</el-radio>
                </el-radio-group>
              </el-form-item>
              <el-form-item label="消息内容" v-if="reply.ReplyType=='Template'" label-position="left">
                <el-input v-model="reply.Template.Content" type="textarea" placeholder="又是元气满满的一天~"/>
              </el-form-item>

              <el-form-item label="AI Model" v-if="reply.ReplyType=='AI'" label-position="left">
                <el-select v-model="reply.OpenAi.Id" placeholder="请选择模型">
                  <template v-for="openai in openai_list">
                    <el-option :label="openai.Name" :value="openai.Id"/>
                  </template>
                </el-select>
              </el-form-item>

              <el-form-item label="AI Prompt" v-if="reply.ReplyType=='AI'" label-position="left">
                <el-input v-model="reply.OpenAi.Prompt" type="textarea" input-style="height:120px"
                          placeholder="请输入Prompt">
                </el-input>
              </el-form-item>
              <el-form-item>
                <el-button type="danger" @click="deleteReply(index,scheduled_rule)"
                           size="small">
                  删除回复
                </el-button>
              </el-form-item>
            </template>
          </el-form>
          <div slot="footer" class="dialog-footer">
            <el-button @click="cancelCreateScheduledRule">取消</el-button>
            <el-button type="primary" @click="createScheduledRule">确定</el-button>
          </div>
        </el-drawer>
        <el-drawer title="编辑规则" v-model="scheduledRuleEditVisible" @close="cancelEditScheduledRule"
                   size="50%"
                   style="padding: 40px 10px 40px 50px" append-to-body>
          <el-form :model="edit_scheduled_rule" label-width="auto" style="max-width: 600px">
            <el-form-item label="规则名称" label-position="left">
              <el-input v-model="edit_scheduled_rule.Name" disabled placeholder="请输入规则名称"/>
            </el-form-item>
            <el-form-item label="人群包" label-position="left">
              <el-select v-model="edit_scheduled_rule.Group.Id" placeholder="请选择人群包">
                <template v-for="group in groups">
                  <el-option :label="group.Name" :value="group.Id"/>
                </template>
              </el-select>
            </el-form-item>
            <el-form-item label="Cron表达式" label-position="left">
              <el-input v-model="edit_scheduled_rule.Cron" placeholder="请输入Cron表达式"/>
            </el-form-item>
            <el-form-item>
              <el-button type="success" @click="createReply(edit_scheduled_rule)"
                         size="small">
                新增回复
              </el-button>
            </el-form-item>
            <template v-for="(reply,index) in edit_scheduled_rule.Reply">
              <el-form-item label="消息类型" label-position="left">
                <el-radio-group v-model="reply.ReplyType">
                  <el-radio value="Template">模板文案</el-radio>
                  <el-radio value="AI">AI</el-radio>
                </el-radio-group>
              </el-form-item>
              <el-form-item label="消息内容" v-if="reply.ReplyType=='Template'" label-position="left">
                <el-input v-model="reply.Template.Content" type="textarea" placeholder="又是元气满满的一天~"/>
              </el-form-item>

              <el-form-item label="AI Model" v-if="reply.ReplyType=='AI'" label-position="left">
                <el-select v-model="reply.OpenAi.Id" placeholder="请选择模型">
                  <template v-for="openai in openai_list">
                    <el-option :label="openai.Name" :value="openai.Id"/>
                  </template>
                </el-select>
              </el-form-item>

              <el-form-item label="AI Prompt" v-if="reply.ReplyType=='AI'" label-position="left">
                <el-input v-model="reply.OpenAi.Prompt"
                          type="textarea"
                          input-style="height:120px"
                          placeholder="请输入Prompt">
                </el-input>
              </el-form-item>
              <el-form-item>
                <el-button type="danger" @click="deleteReply(index,edit_scheduled_rule)"
                           size="small">
                  删除回复
                </el-button>
              </el-form-item>
            </template>
          </el-form>
          <div slot="footer" class="dialog-footer">
            <el-button @click="cancelEditScheduledRule">取消</el-button>
            <el-button type="primary" @click="updateScheduledRule(edit_scheduled_rule)">更新</el-button>
          </div>
        </el-drawer>
      </el-collapse-item>
      <!--      <el-collapse-item title="一键群发" name="5" :icon="CaretRight">-->

      <!--      </el-collapse-item>-->
    </el-collapse>
  </div>
</template>

<style>

</style>

<style scoped>

.table-container {
  overflow-x: auto;
  width: 100%;
}

.rule-container {
  width: 100%;
  padding: 10px;
  display: flex;
  flex-direction: column;
  overflow-x: clip;
}

.avatar {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  margin-right: 5px;
}

.select-member-item {
  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: center;
}

.keyword-button {
  width: 90px;
  line-height: 30px;
  padding-top: 0;
  padding-bottom: 0;
}

.keyword-input {
  width: 90px;
  vertical-align: bottom;
}
</style>