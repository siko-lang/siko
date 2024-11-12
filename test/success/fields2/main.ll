%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Main_Container_Bool_Bool = type { %struct.Bool_Bool }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_Container_Bool_Bool(ptr noundef %item, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Container_Bool_Bool, align 4
   %field0 = getelementptr inbounds %struct.Main_Container_Bool_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %item, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %unit_7 = alloca %struct.siko_Tuple_, align 4
   %b_6 = alloca %struct.Bool_Bool, align 4
   %call_5 = alloca %struct.Bool_Bool, align 4
   %valueRef_4 = alloca %struct.Main_Container_Bool_Bool, align 4
   %a_3 = alloca %struct.Main_Container_Bool_Bool, align 4
   %call_2 = alloca %struct.Main_Container_Bool_Bool, align 4
   %call_1 = alloca %struct.Bool_Bool, align 4
   call void @Bool_Bool_True(ptr %call_1)
   call void @Main_Container_Bool_Bool(ptr %call_1, ptr %call_2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_3, ptr align 4 %call_2, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_4, ptr align 4 %a_3, i64 4, i1 false)
   call void @Main_other_Bool_Bool(ptr %valueRef_4, ptr %call_5)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b_6, ptr align 4 %call_5, i64 4, i1 false)
   call void @siko_Tuple_(ptr %unit_7)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_7, i64 0, i1 false)
   ret void
}

define private void @Main_other_Bool_Bool(ptr noundef %c, ptr noundef %fn_result) {
block0:
   %valueRef_1 = alloca %struct.Main_Container_Bool_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %valueRef_1, ptr align 4 %c, i64 4, i1 false)
   %fieldRef_2 = getelementptr inbounds %struct.Main_Container_Bool_Bool, ptr %valueRef_1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %fieldRef_2, i64 4, i1 false)
   ret void
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_True, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


