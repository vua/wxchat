<script setup lang="ts">
import {globalStore, OpenAiConfig, RedeemResp} from "../stores/global_store.ts";
import {storeToRefs} from "pinia";
import {convertFileSrc, invoke} from "@tauri-apps/api/core";
import {Setting, SwitchButton} from "@element-plus/icons-vue";

import {basicSetup, EditorView} from "codemirror"
import {json} from "@codemirror/lang-json"
import {ref, watch} from "vue";
import {oneDark} from "@codemirror/theme-one-dark"
import {ElInput, ElLoading, ElMessage} from "element-plus";
import moment from 'moment';


const store = globalStore()
const {user, root, openai_config_list, homepage_enable, logout, expired_time} = storeToRefs(store)

const clickUserAvatar = () => {

}

const openai_config_editor = ref<Element>()

// onMounted(() => {
//   nextTick(() => {
//     if (openai_config_editor.value && openai_config_drawer.value) {
//       new EditorView({
//         doc: "hi!",
//         extensions: [basicSetup],
//         parent: openai_config_editor.value
//       })
//     }
//   })
// })

const drawer = ref(false)
const openai_config_drawer = ref(false)
const logout_confirm_dialog = ref(false)
const auth_redeem_drawer = ref(false)

let editor = ref<EditorView | null>(null);

watch(openai_config_editor, (_) => {
  if (!openai_config_editor.value) {
    return;
  }
  if (!editor.value) {
    editor.value = new EditorView({
      doc: JSON.stringify(openai_config_list.value, null, 4),
      extensions: [basicSetup, oneDark, json()],
      parent: openai_config_editor.value
    })
  }
})

const getImgSrc = (name?: string) => {
  return convertFileSrc(`${root.value}/resources/image/avatar/${name}.jpg`)
}

async function updateOpenAiConfig() {
  if (editor.value) {
    let configs = editor.value.state.doc.toString()
    try {
      console.log("OpenAiConfig:", JSON.parse(configs) as OpenAiConfig[])
    } catch (e) {
      ElMessage(
          {
            message: "OpenAiConfig , 更新失败 , 请检测配置格式.",
            type: "warning"
          }
      )
      return
    }

    openai_config_list.value = JSON.parse(await invoke("update_openai_config", {configs: configs})) as OpenAiConfig[]
    ElMessage(
        {
          message: "OpenAiConfig , 更新成功 .",
          type: "success"
        }
    )
  }
}

const redeem = async function () {
  let resp = JSON.parse(await invoke("redeem", {"token": token.value})) as RedeemResp;
  ElMessage(
      {
        message: resp.StatusInfo.StatusMsg,
        type: resp.StatusInfo.StatusCode == 0 ? "success" : "warning"
      }
  )
  expired_time.value = resp.ExpiredTime
  token.value = ""
}


const doLogout = async function () {
  logout_confirm_dialog.value = false
  let logout_instance = ElLoading.service({
    lock: true,
    text: "退出中",
    background: 'rgba(0, 0, 0, 0.7)'
  })
  logout.value = true
  await invoke("logout")
  homepage_enable.value = false
  logout_instance.close()
}
const token = ref("")

</script>

<template>
  <div class="user-container">
    <div class="avatar-container">
      <el-popover
          ref="popover"
          placement="right"
          trigger="hover"
          width="256px"
      >
        <div style="font-size: 12px">权益到期时间：{{ moment(expired_time * 1000).format("YYYY-MM-DD HH:mm:ss") }}</div>
        <template #reference>
          <img class="avatar"
               :src="getImgSrc(user.UserName)"
               alt="" @click="clickUserAvatar">
        </template>
      </el-popover>
    </div>
    <div class="setting-container">
      <el-icon class="setting-item" style="font-size: 30px;color: #6b778c; margin-top: 10px" @click="drawer=true">
        <Setting/>
      </el-icon>
    </div>
    <el-drawer v-model="drawer" size="20%" title="高级设置">
      <div class="setting-list">
        <el-button type="primary" @click="openai_config_drawer = true" style="width: 100%" plain>
          大模型配置
        </el-button>
        <br>
        <el-button type="primary" @click="auth_redeem_drawer = true" style="width: 100%" plain>
          兑换权益
        </el-button>
        <br>
        <el-button type="danger" @click="logout_confirm_dialog = true" style="width: 100%" plain>
          <el-icon>
            <SwitchButton/>
          </el-icon>&nbsp;退出登录
        </el-button>
        <el-drawer
            v-model="openai_config_drawer"
            size="50%"
            title="大模型配置"
            :append-to-body="true"
        >

          <div style="padding: 5px 0">
            <el-button type="primary" @click="updateOpenAiConfig">
              更新
            </el-button>
          </div>
          <div ref="openai_config_editor" id="openai-config-editor"></div>

        </el-drawer>
        <el-drawer
            v-model="auth_redeem_drawer"
            size="50%"
            title="权益兑换"
            :append-to-body="true"
        >
          <el-input style="width: 300px;margin-bottom: 5px" v-model="token" placeholder="请输入兑换码">
            <template #append>
              <el-button type="success" plain @click="redeem">兑换</el-button>
            </template>
          </el-input>
        </el-drawer>
      </div>
    </el-drawer>
    <el-dialog
        v-model="logout_confirm_dialog"
        title="登出确认"
        width="500"
    >
      <span>确认要退出登录吗？</span>
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="logout_confirm_dialog = false">取消</el-button>
          <el-button type="primary" @click="doLogout">
            确认
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.user-container {
  background: rgb(214, 213, 206);
  display: flex;
  justify-content: center;
  width: 60px;
  height: 100vh;
  flex-direction: column;
}

.avatar {
  width: 40px;
  height: 40px;
  border-radius: 5px;
}

.avatar-container {
  padding-top: 20px;
  display: flex;
  align-items: center;
  flex-direction: column;
  height: 50vh;
}

.setting-container {
  display: flex;
  align-items: center;
  flex-direction: column-reverse;
  padding-bottom: 20px;
  height: 50vh;
}

.setting-list {
  display: flex;
  align-items: center;
  flex-direction: column;
  width: 100%;
}

#openai-config-editor {
  width: 100%;
}
</style>