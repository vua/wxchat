<script setup lang="ts">
import {convertFileSrc, invoke} from "@tauri-apps/api/core";
import {ref} from "vue";
import {globalStore, Member, User} from "../stores/global_store.ts";
import {storeToRefs} from "pinia";
import {ElLoading} from 'element-plus';
import 'element-plus/theme-chalk/el-loading.css'
import {LoadingInstance} from "element-plus/es/components/loading/src/loading";
import {Picture as IconPicture} from '@element-plus/icons-vue'
import * as path from '@tauri-apps/api/path';

const store = globalStore()
const {loading_completed, members, user, root} = storeToRefs(store)

const login_code = ref("")

let imgSrc = ref("");

async function getImgSrc() {
  root.value = await path.resourceDir();
  imgSrc.value = convertFileSrc(`${root.value}/resources/image/code/qr_code.jpg`)
}

getImgSrc()

const timestamp = ref(new Date().getTime());

const qr_code_ready = ref(false)

async function refresh() {
  qr_code_ready.value = false
  await invoke("refresh", {});
  timestamp.value = new Date().getTime()
  console.log("更新微信登录二维码")
  qr_code_ready.value = true
}

async function check() {
  login_code.value = await invoke("check", {});
  console.log("登录状态码:", login_code.value)
}

async function load_members() {
  members.value = JSON.parse(await invoke("get_members", {})) as Member[];
  console.log("加载微信联系人")
}

async function load_user() {
  user.value = JSON.parse(await invoke("get_user", {})) as User;
  console.log("加载个人信息")
}

async function start_listen() {
  await invoke("listen", {})
  console.log("开启监听")
}


let loading: LoadingInstance;

let enter = false;

refresh()


let timer = setInterval(function () {
  if (!qr_code_ready.value) {
    return
  }
  check().then(async () => {
    if (login_code.value == "200") {
      clearInterval(timer)
      console.log("清除定时任务")
      await load_user()
      await load_members()
      loading.close()
      loading_completed.value = true
      await start_listen()
    } else if (login_code.value == "201") {
      if (enter) {
        return
      }
      enter = true
      loading = ElLoading.service({
        lock: true,
        text: "加载中",
        background: 'rgba(0, 0, 0, 0.7)'
      })
    }
  })
}, 10000);

</script>

<template>
  <div class="login-container">
    <div style="margin: 5px">
      微信扫码登录
    </div>
    <div class="qr-code">
      <el-image :src="`${imgSrc}?timestamp=${timestamp}`">
        <template #error>
          <div class="image-slot">
            <el-icon>
              <IconPicture/>
            </el-icon>
          </div>
        </template>
      </el-image>
    </div>
  </div>
</template>

<style scoped>
.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
}

.qr-code {
  border: #ccc 1px solid;
  margin: 15px;
}

.qr-code .el-image {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 200px;
  height: 200px;
}

.qr-code .el-icon {
  font-size: 40px;
  color: #cccccc;
}
</style>